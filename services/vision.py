import base64
from io import BytesIO
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, LOCAL_VISION_MODEL
from .downloader import ModelDownloader

import litellm
litellm.set_verbose = False
litellm.suppress_debug_info = True


class BaseVisionService(ABC):
    model: str = "unknown"

    @abstractmethod
    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        pass


class LocalVisionService(BaseVisionService):
    def __init__(self, model_name: str):
        self.model = model_name
        self._model_loaded = False
        self._processor = None
        self._model = None
        self._device = None
        self._is_moondream = "moondream" in model_name.lower()

    def _ensure_model_loaded(self):
        if self._model_loaded:
            return

        print(f"[Lotaria] Loading local vision model: {self.model}")
        
        try:
            import torch
        except OSError as e:
            error_msg = str(e)
            if "DLL" in error_msg or "c10.dll" in error_msg:
                raise RuntimeError(
                    "Local vision model failed to load. This usually means PyTorch is missing DLLs.\n\n"
                    "RECOMMENDATION: Use the 'Ollama (Easiest Local)' provider in Settings instead. "
                    "It is much more compatible with different hardware and easier to set up."
                ) from e
            raise

        # Model mapping for better compatibility
        downloader = ModelDownloader()
        huggingface_id = self.model
        
        # Handle "Custom" or specific aliases
        if "moondream2" in self.model:
            huggingface_id = "vikhyatk/moondream2"
            self._is_moondream = True
        elif "qwen3-vl" in self.model:
            huggingface_id = "Qwen/Qwen2.5-VL-3B-Instruct"
        elif "custom" in self.model:
            # Look for custom model ID in config
            huggingface_id = config.get("custom_settings", {}).get("local_vision_id")
            if not huggingface_id:
                raise ValueError("Custom model selected but no 'local_vision_id' set in settings.")
            self._is_moondream = "moondream" in huggingface_id.lower()

        # Download if needed
        model_path = downloader.ensure_model(huggingface_id)
        print(f"[Lotaria] Using model files from: {model_path}")

        try:
            from transformers import AutoProcessor, AutoModelForCausalLM, AutoModelForVision2Seq
            
            self._processor = AutoProcessor.from_pretrained(huggingface_id, trust_remote_code=True)
            
            model_kwargs = {
                "trust_remote_code": True,
                "device_map": "auto",
            }
            
            # Use float16/bfloat16 for efficiency on GPU
            if torch.cuda.is_available():
                model_kwargs["torch_dtype"] = torch.bfloat16 if torch.cuda.is_bf16_supported() else torch.float16

            if self._is_moondream:
                self._model = AutoModelForCausalLM.from_pretrained(huggingface_id, **model_kwargs)
            else:
                self._model = AutoModelForVision2Seq.from_pretrained(huggingface_id, **model_kwargs)
                
        except Exception as e:
            print(f"[Lotaria] Error loading model {huggingface_id}: {e}")
            raise

        self._device = next(self._model.parameters()).device
        self._model_loaded = True
        print(f"[Lotaria] Vision model loaded on {self._device}")

    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        self._ensure_model_loaded()
        from PIL import Image
        import torch

        img = Image.open(BytesIO(image_bytes)).convert("RGB")

        if self._is_moondream:
            # Moondream specific inference
            return self._model.answer_question(self._processor(img), prompt, self._processor)

        messages = [
            {
                "role": "user",
                "content": [
                    {"type": "image", "image": img},
                    {"type": "text", "text": prompt},
                ],
            }
        ]

        text = self._processor.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True
        )
        inputs = self._processor(
            text=[text], images=[img], padding=True, return_tensors="pt",
        ).to(self._device)

        with torch.no_grad():
            generated_ids = self._model.generate(
                **inputs, max_new_tokens=256, do_sample=True, temperature=0.7,
            )

        generated_ids_trimmed = [
            out_ids[len(in_ids):]
            for in_ids, out_ids in zip(inputs.input_ids, generated_ids)
        ]
        output_text = self._processor.batch_decode(
            generated_ids_trimmed, skip_special_tokens=True,
            clean_up_tokenization_spaces=False,
        )[0]

        return output_text.strip()


class LiteLLMVisionService(BaseVisionService):
    def __init__(self, model: str, api_key: str = None):
        self.model = model
        self.api_key = api_key

    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        b64 = base64.b64encode(image_bytes).decode()
        kwargs = {
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b64}"}},
                ],
            }],
        }
        if self.api_key:
            kwargs["api_key"] = self.api_key
        response = litellm.completion(**kwargs)
        return response.choices[0].message.content


class OllamaVisionService(BaseVisionService):
    """Uses Ollama's local API. Most straightforward setup."""
    def __init__(self, model: str):
        # ollama/moondream -> moondream
        self.model = model.split("/")[-1]
        if self.model == "custom":
            self.model = config.get("custom_settings", {}).get("ollama_vision_id", "moondream")

    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        import requests
        import base64
        
        b64 = base64.b64encode(image_bytes).decode()
        url = config.get("custom_settings", {}).get("ollama_url", "http://localhost:11434/api/generate")
        
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "images": [b64]
        }
        
        try:
            # First load can take a while, 180s is safer
            response = requests.post(url, json=payload, timeout=180)
            response.raise_for_status()
            return response.json().get("response", "").strip()
        except requests.exceptions.Timeout:
            return f"[Ollama Timeout] The model '{self.model}' is taking too long to load/respond. Try a smaller/quantized model or wait for it to initialize in Ollama."
        except Exception as e:
            return f"[Ollama Error] Ensure Ollama is running and '{self.model}' is pulled. ({e})"


class LMStudioVisionService(BaseVisionService):
    """Uses LM Studio's OpenAI-compatible API."""
    def __init__(self, model: str):
        # lmstudio/custom -> custom
        self.model = model.split("/")[-1]
        if self.model == "custom":
            self.model = config.get("custom_settings", {}).get("lmstudio_vision_id", "model-identifier")

    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        import requests
        import base64
        
        b64 = base64.b64encode(image_bytes).decode()
        url = config.get("custom_settings", {}).get("lmstudio_url", "http://localhost:1234/v1/chat/completions")
        
        payload = {
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {"type": "text", "text": prompt},
                        {
                            "type": "image_url",
                            "image_url": {"url": f"data:image/jpeg;base64,{b64}"}
                        }
                    ]
                }
            ],
            "max_tokens": 300
        }
        
        try:
            # First load can take a while, 180s is safer
            response = requests.post(url, json=payload, timeout=180)
            response.raise_for_status()
            return response.json()["choices"][0]["message"]["content"].strip()
        except requests.exceptions.Timeout:
            return f"[LM Studio Timeout] The model is taking too long to load. Ensure you have 'GPU Offloading' enabled in LM Studio for better speed."
        except Exception as e:
            return f"[LM Studio Error] Ensure LM Studio server is running on {url}. ({e})"


# ---------------------------------------------------------------------------
# Factory (lazy singleton, recreated when model/provider changes)
# ---------------------------------------------------------------------------

_vision_service: Optional[BaseVisionService] = None
_vision_service_key: Optional[str] = None


def get_vision_service() -> BaseVisionService:
    global _vision_service, _vision_service_key

    provider = config["vision_provider"]
    model = config["vision_model"]
    current_key = f"{provider}:{model}"

    if _vision_service is not None and _vision_service_key != current_key:
        _vision_service = None

    if _vision_service is None:
        if provider == "local":
            _vision_service = LocalVisionService(model_name=model)
        elif provider == "ollama":
            _vision_service = OllamaVisionService(model=model)
        elif provider == "lmstudio":
            _vision_service = LMStudioVisionService(model=model)
        else:
            api_key = config.get("api_keys", {}).get(provider)
            _vision_service = LiteLLMVisionService(model=model, api_key=api_key)
        _vision_service_key = current_key
        print(f"[Lotaria] Vision service initialized: {model}")

    return _vision_service
