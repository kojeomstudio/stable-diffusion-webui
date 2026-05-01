# ROCm (AMD GPU) 호환성 기술 검토 보고서

> 작성일: 2026-05-01  
> 대상: AUTOMATIC1111/stable-diffusion-webui (kojeomstudio fork)  
> 검토자: Kilo AI Agent  

---

## 1. 핵심 결론

**ROCm으로 동작 가능 — 단, 환경변수 설정 및 일부 CLI 플래그 조합 필요.**  
네이티브 ROCm 지원 코드는 현재 없으나, PyTorch ROCm 빌드가 `torch.cuda` API를 호환 제공하므로 대부분의 기능이 자동 작동함.

---

## 2. ROCm 작동 원리: CUDA API 호환 레이어

PyTorch ROCm(HIP) 빌드에서는 CUDA API가 투명하게 매핑됨:

| CUDA API | ROCm 동작 |
|----------|-----------|
| `torch.cuda.is_available()` | `True` 반환 |
| `torch.cuda.get_device_name()` | AMD GPU 이름 반환 |
| `torch.cuda.empty_cache()` | 정상 동작 (HIP으로 매핑) |
| `torch.cuda.memory_stats()` | 정상 동작 |
| `device.type` | `"cuda"` 반환 |
| `torch.autocast("cuda")` | ROCm fp16 autocast 지원 |
| `torch.cuda.get_device_capability()` | **주의**: 예상과 다른 값 반환 가능 |

즉, `modules/devices.py`의 장치 추상화가 ROCm을 자동으로 "CUDA"로 인식함.

---

## 3. 기존 비CUDA 백엔드 지원 현황 (참고)

| 백엔드 | 모듈 | 장치 타입 | `torch.cuda` 호환 |
|--------|------|-----------|-------------------|
| MPS (Apple) | `mac_specific.py` | `"mps"` | 아니오 — 전용 API |
| XPU (Intel) | `xpu_specific.py` | `"xpu"` | 아니오 — 전용 API |
| NPU (Huawei) | `npu_specific.py` | `"npu"` | 아니오 — 전용 API |
| **ROCm (AMD)** | **없음** | **`"cuda"`** | **예** — CUDA API 호환 |

---

## 4. 문제 영역 상세 분석

### 4.1 [HIGH] xformers — CUDA 전용, ROCm 작동 불가

**파일**: `modules/sd_hijack_optimizations.py:57`

```python
torch.cuda.is_available() and (6, 0) <= torch.cuda.get_device_capability(shared.device) <= (9, 0)
```

- `get_device_capability()`는 ROCm에서 예상치 못한 값 반환
- xformers 자체가 CUDA 커널을 직접 컴파일하므로 ROCm에서 빌드 불가
- **대안**: `--opt-sdp-attention` (PyTorch 내장 SDPA) 사용

### 4.2 [HIGH] `torch.cuda.get_device_capability()` — ROCm 오동작 가능

**사용 위치**:
- `modules/devices.py:30` — GTX 16xx autocast 체크
- `modules/sd_hijack_optimizations.py:57` — xformers 가용성 체크

ROCm에서 반환값이 다를 수 있어 비교 로직에서 예외 발생 위험.

### 4.3 [MEDIUM] `torch.backends.cuda.sdp_kernel()` — ROCm 미지원 가능

**파일**: `modules/sd_hijack_optimizations.py:550, 659` (sdp-no-mem 경로)

```python
with torch.backends.cuda.sdp_kernel(enable_flash=True, ...):
```

Flash Attention 커널 제어용으로, ROCm 지원 여부 불확실.  
`--opt-sdp-no-mem-attention` 사용 시 문제 가능.

### 4.4 [MEDIUM] SD3 모델 — 하드코딩된 autocast 데코레이터

**파일**: `modules/models/sd3/sd3_impls.py:151, 364, 368`

```python
@torch.autocast("cuda", dtype=torch.float16)
```

ROCm에서는 `"cuda"` 디바이스로 autocast가 작동하므로 **호환됨**.  
단, ROCm 버전에 따라 동작 차이 가능.

### 4.5 [LOW] `torch.cuda.amp.GradScaler()` — 훈련 전용

**파일**: `modules/textual_inversion/textual_inversion.py:493`, `modules/hypernetworks/hypernetwork.py:568`

훈련 기능에서만 사용. 추론에는 영향 없음.

### 4.6 [LOW] `enable_tf32()` — ROCm에서 no-op

**파일**: `modules/devices.py:101-110`

cuDNN/TF32 설정은 ROCm에 해당하지 않아 자동 무시됨.

### 4.7 [LOW] `MemUsageMonitor` — 자동 비활성화

**파일**: `modules/memmon.py:28`

```python
except Exception as e:  # AMD or whatever
```

ROCm에서 메모리 API가 작동하면 정상 동작, 아니면 자동 비활성화.

---

## 5. 호환성 매트릭스

| 기능 | CUDA | ROCm | 비고 |
|------|------|------|------|
| 기본 추론 (txt2img, img2img) | O | O | `torch.cuda` API 호환 |
| fp16 (half precision) | O | O | ROCm fp16 지원 |
| bf16 (bfloat16) | O | Delta | RDNA3+ 에서만 |
| fp8 양자화 | O | X | ROCm FP8 미지원 |
| xformers | O | X | CUDA 전용 커널 |
| SDPA (PyTorch 내장) | O | O | ROCm 5.4+ |
| sdp-no-mem | O | Delta | `sdp_kernel` 호환성 불확실 |
| sub-quadratic attention | O | O | Pure PyTorch |
| 모델 로딩 | O | O | 디바이스 무관 |
| VAE 디코딩 | O | O | `--no-half-vae` 권장 |
| LoRA | O | O | 디바이스 무관 |
| Hypernetwork 훈련 | O | Delta | `GradScaler` 호환성 |
| Textual Inversion 훈련 | O | Delta | `GradScaler` 호환성 |
| CLIP Interrogate | O | O | 디바이스 무관 |
| GFPGAN / CodeFormer | O | O | 디바이스 무관 |
| ESRGAN / SwinIR 업스케일 | O | O | 디바이스 무관 |
| SD3 모델 | O | Delta | 테스트 필요 |
| 메모리 모니터링 | O | Delta | 자동 비활성화 가능 |

> O = 작동, X = 작동 안함, Delta = 제한적/테스트 필요

---

## 6. 권장 실행 명령어

```bash
export TORCH_INDEX_URL=https://download.pytorch.org/whl/rocm5.7

python launch.py \
  --no-half-vae \
  --opt-sdp-attention \
  --skip-torch-cuda-test
```

| 플래그 | 목적 |
|--------|------|
| `TORCH_INDEX_URL` | PyTorch ROCm wheel 다운로드 |
| `--no-half-vae` | ROCm fp16 VAE에서 NaN 발생 방지 |
| `--opt-sdp-attention` | xformers 대체 (PyTorch 내장 SDPA) |
| `--skip-torch-cuda-test` | ROCm 드라이버 초기화 실패 시에도 시작 |

---

## 7. 네이티브 ROCm 지원 추가 로드맵 (선택사항)

기존 MPS/XPU/NPU와 동일한 패턴으로 ROCm 전용 지원 추가 가능.

### 7.1 필요 파일/수정

| 작업 | 파일 | 설명 |
|------|------|------|
| 신규 생성 | `modules/rocm_specific.py` | AMD GPU 감지, VRAM 쿼리, 워크어라운드 |
| 수정 | `modules/cmd_args.py` | `--use-rocm` CLI 인수 추가 |
| 수정 | `modules/launch_utils.py` | ROCm wheel URL 자동 설정, CUDA 테스트 스킵 |
| 수정 | `modules/devices.py` | ROCm 감지 로직, `get_optimal_device_name()` 업데이트 |
| 수정 | `modules/sd_hijack_optimizations.py` | xformers 가용성 체크에 ROCm 필터링 |
| 수정 | `modules/sd_models.py` | FP8 비활성화 조건에 ROCm 추가 |

### 7.2 `rocm_specific.py` 구조 (참고)

MPS/XPU/NPU 모듈과 동일한 인터페이스:

```python
# modules/rocm_specific.py
import torch

def check_for_rocm() -> bool:
    if not torch.cuda.is_available():
        return False
    name = torch.cuda.get_device_name(0)
    return "AMD" in name or "Radeon" in name

has_rocm = check_for_rocm()

def torch_rocm_gc():
    if has_rocm:
        torch.cuda.empty_cache()
        torch.cuda.ipc_collect()
```

---

## 8. 지원 GPU 참고

ROCm 공식 지원 AMD GPU (Linux 전용, Windows는 미지원):

- RX 7900 XTX, RX 7900 XT, RX 7900 GRE (RDNA3)
- RX 7800 XT, RX 7700 XT (RDNA3)
- RX 6800 XT, RX 6800, RX 6700 XT (RDNA2)
- RX 7600, RX 6600 XT (제한적 지원)
- Instinct MI210, MI250, MI300 (데이터센터)

> **주의**: ROCm은 현재 Linux 전용입니다. Windows에서는 DirectML 또는 WSL2를 통해서만 AMD GPU를 사용할 수 있습니다.

---

## 9. 관련 코드 참조

| 영역 | 파일 | 라인 |
|------|------|------|
| 장치 추상화 | `modules/devices.py` | 50-63 (우선순위 체인) |
| autocast 분기 | `modules/devices.py` | 210-231 |
| GC (가비지컬렉션) | `modules/devices.py` | 77-92 |
| xformers 가용성 | `modules/sd_hijack_optimizations.py` | 56-57 |
| CUDA 테스트 | `modules/launch_utils.py` | 409-415 |
| Torch wheel 선택 | `modules/launch_utils.py` | 318-340 |
| SDPA 커널 | `modules/sd_hijack_optimizations.py` | 550, 659 |
| SD3 autocast | `modules/models/sd3/sd3_impls.py` | 151, 364, 368 |
