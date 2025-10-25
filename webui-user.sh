#!/usr/bin/env bash
# ===============================================
# macOS (Apple Silicon)용 webui-user.sh (강제 3.12)
#  - pyenv 3.12.9 또는 Homebrew python@3.12만 허용
#  - 시스템 python3(=3.13)로는 절대 폴백하지 않음
# ===============================================

# 1) 후보 경로 지정
PYENV_PY="$HOME/.pyenv/versions/3.10.13/bin/python3"

# 2) 우선순위: pyenv 3.10.13 > Homebrew 3.12
if [ -x "$PYENV_PY" ]; then
  python_cmd="$PYENV_PY"
else
  echo "[ERROR] Python 3.10.13가 발견되지 않았습니다."
  echo "        아래 중 하나를 먼저 준비해 주세요:"
  echo "        - pyenv:  pyenv install 3.10.13"
  echo "        - brew:   brew install python@3.10"
  exit 1
fi

# 가시적 로그
echo "[INFO] Using python_cmd=$python_cmd"
"$python_cmd" -V || true

# 3) macOS MPS(Apple GPU) 폴백 허용
export PYTORCH_ENABLE_MPS_FALLBACK=1

# 4) 실행 옵션 (macOS 기본 안정 구성)
#  - --no-half: MPS에서 FP16 이슈 회피 (필요 없으면 제거 가능)
#  - --opt-sdp-attention: 메모리 효율 어텐션
#  - --api: API 사용
export COMMANDLINE_ARGS="--api --skip-torch-cuda-test"

# 5) (선택) Torch 버전 고정 (보통 자동 설치되므로 주석 유지)
# export TORCH_COMMAND="pip install torch==2.3.1 torchvision==0.18.1"

# ===============================================