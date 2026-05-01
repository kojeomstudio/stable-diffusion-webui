# 디펜던시 감사 보고서

> 작성일: 2026-05-01
> 대상: AUTOMATIC1111/stable-diffusion-webui (kojeomstudio fork)
> 목적: 포크/대체된 리포지터리 현황 정리 및 누락 이슈 점검

---

## 1. Git 클론 리포지터리 현황

`modules/launch_utils.py`에서 `git_clone()`으로 클론하는 5개 리포지터리.

### 1.1 Assets

| 항목 | 값 |
|------|-----|
| 원본 | `AUTOMATIC1111/stable-diffusion-webui-assets` |
| 포크 | `kojeomstudio/stable-diffusion-webui-assets` |
| 커밋 | `6f7db241d2f8ba7457bac5ca9753331f0c266917` |
| 원본 상태 | **정상 유지** (활성) |
| 포크 상태 | 정상 |
| 이슈 | 없음 |

### 1.2 Stable Diffusion (CompVis/Stability-AI)

| 항목 | 값 |
|------|-----|
| 원본 | `Stability-AI/stablediffusion` |
| 포크 | `kojeomstudio/stable-diffusion-v2` (nlile/stablediffusion 기반) |
| 커밋 | `ef2b0ce8fb8265e940d509964dfdc0631b74b21a` |
| 원본 상태 | **삭제됨 (HTTP 404)** |
| 포크 상태 | 정상 |
| 이슈 | **[해결됨]** 원본이 삭제되어 포크로 대체. 커밋 해시도 포크 기준으로 변경 완료. |

### 1.3 Stable Diffusion XL / Generative Models

| 항목 | 값 |
|------|-----|
| 원본 | `Stability-AI/generative-models` |
| 포크 | `kojeomstudio/generative-models` |
| 커밋 | `45c443b316737a4ab6e40413d7794a7f5657c19f` |
| 원본 상태 | **정상 유지** (27.1k stars, 활성) |
| 포크 상태 | 정상 (upstream 동기화됨) |
| 이슈 | 없음 |

### 1.4 K-Diffusion

| 항목 | 값 |
|------|-----|
| 원본 | `crowsonkb/k-diffusion` |
| 포크 | `kojeomstudio/k-diffusion` |
| 커밋 | `ab527a9a6d347f364e3d185ba6d714e22d80cb3c` |
| 원본 상태 | **정상 유지** (2.6k stars, 활성) |
| 포크 상태 | 정상 (upstream 1 커밋 뒤처짐) |
| 이슈 | 없음. 최신이 아닌 고정 커밋 사용은 의도적. |

### 1.5 BLIP

| 항목 | 값 |
|------|-----|
| 원본 | `salesforce/BLIP` |
| 포크 | `kojeomstudio/BLIP` |
| 커밋 | `48211a1594f1321b00f14c9f7a5b4813144b2fb9` |
| 원본 상태 | **Archived (읽기전용, Deprecated)** |
| 포크 상태 | 정상 |
| 이슈 | **[해결됨]** 원본이 Archived되어 포크로 보존. |

---

## 2. Pip 패키지 (launch_utils.py에서 설치)

### 2.1 PyTorch + TorchVision

| 항목 | 값 |
|------|-----|
| 기본 | `torch==2.1.2`, `torchvision==0.16.2` (CUDA 12.1) |
| macOS ARM | `torch==2.3.1`, `torchvision==0.18.1` |
| IPEX Windows | torch 2.0.0a0 (Nuullll 커스텀 빌드) |
| IPEX Linux | torch 2.0.0a0 + intel-extension-for-pytorch |
| 환경변수 | `TORCH_COMMAND`, `TORCH_INDEX_URL` 로 오버라이드 가능 |

### 2.2 CLIP

| 항목 | 값 |
|------|-----|
| 원본 URL | `github.com/openai/CLIP/archive/d50d76da...zip` |
| 현재 URL | `github.com/kojeomstudio/stable-diffusion-webui/raw/.../archive/CLIP-d50d76da...zip` |
| 원본 상태 | **정상** (openai/CLIP 활성, 33k+ stars) |
| 설치 방식 | 로컬 `archive/` 폴더에서 self-hosted zip |
| 이슈 | 없음. 원본은 아직 활성이나, self-hosting으로 더 안정적. |

### 2.3 OpenCLIP

| 항목 | 값 |
|------|-----|
| 원본 URL | `github.com/mlfoundations/open_clip/archive/bb6e834e...zip` |
| 현재 URL | `github.com/kojeomstudio/stable-diffusion-webui/raw/.../archive/open_clip-bb6e834e...zip` |
| 원본 상태 | **정상, 매우 활성** (13.8k stars, main 브랜치에 Breaking changes 존재) |
| 설치 방식 | 로컬 `archive/` 폴더에서 self-hosted zip |
| 이슈 | 없음. 원본 main 브랜치가 빠르게 변경되므로 고정 커밋 사용이 올바른 접근. |

### 2.4 xformers

| 항목 | 값 |
|------|-----|
| 버전 | `xformers==0.0.23.post1` |
| 설치 조건 | `--xformers` 플래그 설정 시에만 |
| 상태 | **CUDA 전용**. ROCm/MPS에서 사용 불가. |

### 2.5 setuptools / wheel (CLIP 호환성)

| 항목 | 값 |
|------|-----|
| 버전 | `setuptools<70.0.0`, `wheel` |
| 목적 | 구버전 CLIP/OpenCLIP 빌드 호환성 |

---

## 3. 런타임 모델 다운로드 URL

런타임에 최초 사용 시 다운로드하는 모델 가중치 파일들.
이들은 **Git 리포지터리가 아닌 Release/Raw 바이너리**이므로 포크 대상이 아님.

### 3.1 정상 작동 (활성 원본)

| 모델 | 원본 URL | 상태 |
|------|----------|------|
| TAESD VAE | `github.com/madebyollin/taesd/raw/main/...` | 정상 |
| VAE Approx | `github.com/AUTOMATIC1111/stable-diffusion-webui/releases/...` | 정상 |
| GFPGAN v1.4 | `github.com/TencentARC/GFPGAN/releases/...` | 정상 |
| CodeFormer | `github.com/sczhou/CodeFormer/releases/...` | 정상 |
| ESRGAN | `github.com/cszn/KAIR/releases/...` | 정상 |
| RealESRGAN | `github.com/xinntao/Real-ESRGAN/releases/...` | 정상 |
| SwinIR | `github.com/JingyunLiang/SwinIR/releases/...` | 정상 |
| ScuNET | `github.com/cszn/KAIR/releases/...` | 정상 |
| DAT Upscaler | `github.com/n0kovo/dat_upscaler_models/raw/...` | 정상 |
| YuNet Face | `github.com/opencv/opencv_zoo/blob/...?raw=true` | 정상 |
| DeepDanbooru | `github.com/AUTOMATIC1111/TorchDeepDanbooru/releases/...` | 정상 |
| MiDaS/DPT | `github.com/intel-isl/DPT/releases/...`, `github.com/AlexeyAB/MiDaS/releases/...` | 정상 |
| BLIP Caption | `storage.googleapis.com/sfr-vision-language-research/BLIP/...` | 정상 (Google Storage) |

### 3.2 잠재적 위험 (모니터링 필요)

| 모델 | URL | 리포 | 위험도 |
|------|-----|------|--------|
| IPEX Windows | `github.com/Nuullll/intel-extension-for-pytorch/releases/...` | 개인 계정 | 중간 — 개인이 관리하는 커스텀 빌드 |
| MiDaS (AlexeyAB) | `github.com/AlexeyAB/MiDaS/releases/...` | 개인 계정 | 낮음 — 과거부터 안정적 |
| n0kovo/dat_upscaler_models | `github.com/n0kovo/dat_upscaler_models/raw/...` | 개인 계정 | 낮음 — 단순 파일 호스팅 |

---

## 4. README에 명시된 포크 중 launch_utils.py에 미사용 리포지터리

README에 Forked Repos로 나열되어 있으나 `launch_utils.py`에서 직접 클론하지 않는 리포지터리:

| 리포지터리 | 용도 | 미사용 사유 |
|------------|------|-------------|
| `kojeomstudio/stable-diffusion` (v1) | SD 1.x 참조용 | v2 포크로 대체됨 |
| `kojeomstudio/taming-transformers` | VQGAN 토크나이저 | BLIP/stablediffusion 서브디펜던시로 간접 사용 가능 |
| `kojeomstudio/CLIP` | CLIP 소스코드 | pip zip으로 설치하므로 직접 클론 불필요 |
| `kojeomstudio/open_clip` | OpenCLIP 소스코드 | pip zip으로 설치하므로 직접 클론 불필요 |

이 리포지터리들은 향후 수동 설치나 디버깅 시 필요할 수 있으므로 **유지 권장**.

---

## 5. 버전 노후화 현황 (requirements_versions.txt)

현재 코드와의 호환성을 위해 의도적으로 핀된 구버전 패키지들.
**업그레이드 시 대규모 코드 수정이 동반됨.**

| 패키지 | 현재 버전 | 최신 버전 | 위험도 | 비고 |
|--------|-----------|-----------|--------|------|
| gradio | 3.41.2 | 5.x | **높음** | UI 전체 재작성 필요. 의도적 핀. |
| transformers | 4.30.2 | 4.45+ | 중간 | 최신 모델 아키텍처 미지원 가능 |
| pytorch_lightning | 1.9.4 | 2.x | 중간 | API 변경. 의도적 핀. |
| fastapi | 0.94.0 | 0.115+ | 중간 | Breaking changes 존재 |
| Pillow | 9.5.0 | 11.x | 낮음 | 보안 패치 누락 가능 |
| protobuf | 3.20.0 | 5.x | 낮음 | TF/ONNX 호환성 위해 의도적 핀 |
| open-clip-torch | 2.20.0 | 3.x | 낮음 | zip 설치 버전과 충돌 가능성 |
| numpy | 1.26.2 | 2.x | 낮음 | numpy 2.x는 Breaking changes 많음 |
| accelerate | 0.21.0 | 1.x | 낮음 | 기능 제한적 사용 |

---

## 6. 이슈 요약 및 권고사항

### 6.1 이미 해결된 이슈 (포크로 대체 완료)

1. **`Stability-AI/stablediffusion` 삭제** → `kojeomstudio/stable-diffusion-v2` 로 대체
2. **`salesforce/BLIP` Archived** → `kojeomstudio/BLIP` 로 보존
3. **CLIP 빌드 실패** → self-hosted zip + setuptools 다운그레이드로 해결 (`docs/CLIP_INSTALLATION_FIX.md`)
4. **OpenCLIP 호환성** → 특정 커밋으로 고정하여 Breaking changes 회피

### 6.2 현재 이슈 없음 (정상 유지)

- `AUTOMATIC1111/stable-diffusion-webui-assets` — 정상
- `Stability-AI/generative-models` — 정상
- `crowsonkb/k-diffusion` — 정상
- `openai/CLIP` — 정상
- `mlfoundations/open_clip` — 정상
- 모든 런타임 모델 다운로드 URL — 정상

### 6.3 모니터링 권고

| 항목 | 이유 | 권고 |
|------|------|------|
| `kojeomstudio/k-diffusion` | upstream 1커밋 뒤처짐 | 필요시 동기화 |
| `Nuullll/intel-extension-for-pytorch` | 개인 계정의 커스텀 빌드 | IPEX 사용 시 의존성 확인 |
| `n0kovo/dat_upscaler_models` | 개인 계정 | DAT 모델 사용 시 확인 |
| 버전 노후화 | gradio, transformers 등 | 장기적으로 업그레이드 계획 수립 |

### 6.4 누락된 디펜던시 없음

현재 `launch_utils.py`, `requirements_versions.txt`, `requirements.txt`, 런타임 다운로드 URL을 모두 검토했으며, **사용자가 놓친 디펜던시 이슈는 발견되지 않았음.** 모든 원본 리포지터리의 삭제/Archived 이슈는 포크로 적절히 대체되었음.

---

## 7. 참고: 전체 외부 URL 맵

### Git 클론 (5개)
- `kojeomstudio/stable-diffusion-webui-assets`
- `kojeomstudio/stable-diffusion-v2`
- `kojeomstudio/generative-models`
- `kojeomstudio/k-diffusion`
- `kojeomstudio/BLIP`

### Pip 설치 (3개 소스)
- PyTorch (torch + torchvision) — pytorch.org wheel
- CLIP — self-hosted zip (archive/)
- OpenCLIP — self-hosted zip (archive/)

### 런타임 모델 다운로드 (13개 소스)
- madebyollin/taesd (TAESD)
- AUTOMATIC1111/stable-diffusion-webui releases (VAE approx)
- TencentARC/GFPGAN releases (GFPGAN)
- sczhou/CodeFormer releases (CodeFormer)
- cszn/KAIR releases (ESRGAN, ScuNET)
- xinntao/Real-ESRGAN releases (RealESRGAN)
- JingyunLiang/SwinIR releases (SwinIR)
- n0kovo/dat_upscaler_models (DAT)
- opencv/opencv_zoo (YuNet)
- AUTOMATIC1111/TorchDeepDanbooru releases (DeepDanbooru)
- intel-isl/DPT releases (MiDaS DPT)
- AlexeyAB/MiDaS releases (MiDaS)
- Google Storage (BLIP caption model)
