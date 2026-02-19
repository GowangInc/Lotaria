import base64
from io import BytesIO
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, LOCAL_VISION_MODEL


class BaseVisionService(ABC):
    model: str = "unknown"

    @abstractmethod
    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        pass


class LocalVisionService(BaseVisionService):
    def __init__(self):
        self.model = LOCAL_VISION_MODEL
        self._model_loaded = False
        self._processor = None
        self._model = None
        self._device = None

    def _ensure_model_loaded(self):
        if self._model_loaded:
            return

        print(f"[Lotaria] Loading local vision model: {self.model}")
        import torch

        try:
            from modelscope import AutoProcessor, AutoModelForVision2Seq
            print(f"[Lotaria] Using ModelScope for model download: {self.model}")

            self._processor = AutoProcessor.from_pretrained(
                self.model, trust_remote_code=True,
            )
            self._model = AutoModelForVision2Seq.from_pretrained(
                self.model, trust_remote_code=True,
                torch_dtype=torch.bfloat16, device_map="auto",
            )
        except ImportError:
            print("[Lotaria] ModelScope not available, falling back to HuggingFace")
            from transformers import AutoProcessor, Qwen2_5_VLForConditionalGeneration

            self._processor = AutoProcessor.from_pretrained(
                self.model, trust_remote_code=True,
            )
            self._model = Qwen2_5_VLForConditionalGeneration.from_pretrained(
                self.model, trust_remote_code=True,
                torch_dtype=torch.bfloat16, device_map="auto",
            )

        self._device = next(self._model.parameters()).device
        self._model_loaded = True
        print(f"[Lotaria] Vision model loaded on {self._device}")

    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        self._ensure_model_loaded()
        from PIL import Image
        import torch

        img = Image.open(BytesIO(image_bytes)).convert("RGB")

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
        import litellm
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
            _vision_service = LocalVisionService()
        else:
            api_key = config.get("api_keys", {}).get(provider)
            _vision_service = LiteLLMVisionService(model=model, api_key=api_key)
        _vision_service_key = current_key
        print(f"[Lotaria] Vision service initialized: {model}")

    return _vision_service
