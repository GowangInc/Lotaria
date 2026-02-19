import os
import base64
from io import BytesIO
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, LOCAL_VISION_MODEL, API_VISION_MODEL


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


class APIVisionService(BaseVisionService):
    def __init__(self):
        from google import genai
        api_key = os.environ.get("API_KEY") or os.environ.get("GOOGLE_API_KEY")
        if not api_key:
            raise ValueError("API_KEY or GOOGLE_API_KEY must be set for API vision model")
        self.client = genai.Client(api_key=api_key)
        self.model = API_VISION_MODEL

    def analyze(self, image_bytes: bytes, prompt: str) -> str:
        image_data = base64.b64encode(image_bytes).decode("utf-8")
        response = self.client.models.generate_content(
            model=self.model,
            contents=[
                {
                    "parts": [
                        {"text": prompt},
                        {
                            "inline_data": {
                                "mime_type": "image/png",
                                "data": image_data,
                            }
                        },
                    ]
                }
            ],
        )
        return response.text


# ---------------------------------------------------------------------------
# Factory (lazy singleton, recreated when model type changes)
# ---------------------------------------------------------------------------

_vision_service: Optional[BaseVisionService] = None


def get_vision_service() -> BaseVisionService:
    global _vision_service

    model_type = config["vision_model_type"]

    if _vision_service is not None:
        current_is_local = isinstance(_vision_service, LocalVisionService)
        if current_is_local != (model_type == "local"):
            _vision_service = None

    if _vision_service is None:
        if model_type == "local":
            _vision_service = LocalVisionService()
        else:
            _vision_service = APIVisionService()
        print(f"[Lotaria] Vision service initialized: {_vision_service.model}")

    return _vision_service
