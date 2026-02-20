import os
import shutil
from pathlib import Path
from .state import MODELS_DIR

class ModelDownloader:
    """Handles downloading models from HuggingFace with ModelScope fallback."""
    
    @staticmethod
    def get_model_path(repo_id: str) -> Path:
        """Returns the local path where a model should be stored."""
        # Clean repo name for folder use (e.g. vikhyatk/moondream2 -> vikhyatk_moondream2)
        safe_name = repo_id.replace("/", "_")
        return MODELS_DIR / safe_name

    def download_hf(self, repo_id: str, **kwargs) -> Path:
        """Download from HuggingFace."""
        from huggingface_hub import snapshot_download
        print(f"[Lotaria] Attempting download from HuggingFace: {repo_id}")
        local_dir = self.get_model_path(repo_id)
        return Path(snapshot_download(repo_id=repo_id, local_dir=local_dir, **kwargs))

    def download_modelscope(self, repo_id: str, **kwargs) -> Path:
        """Download from ModelScope (China fallback)."""
        from modelscope.hub.snapshot_download import snapshot_download
        print(f"[Lotaria] Attempting download from ModelScope: {repo_id}")
        local_dir = self.get_model_path(repo_id)
        return Path(snapshot_download(repo_id, local_dir=local_dir, **kwargs))

    def ensure_model(self, repo_id: str, **kwargs) -> Path:
        """Ensure model exists locally, downloading if necessary."""
        local_dir = self.get_model_path(repo_id)
        
        # Check if already exists and looks like a model (has config or bin)
        if local_dir.exists() and any(local_dir.iterdir()):
            return local_dir

        try:
            return self.download_hf(repo_id, **kwargs)
        except Exception as e:
            print(f"[Lotaria] HuggingFace download failed: {e}")
            try:
                return self.download_modelscope(repo_id, **kwargs)
            except Exception as ms_e:
                print(f"[Lotaria] ModelScope download failed: {ms_e}")
                raise RuntimeError(f"Could not download model {repo_id} from any source.")
