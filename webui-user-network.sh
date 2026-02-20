#!/usr/bin/env bash
# ===============================================
# macOS (Apple Silicon)용 webui-user-network.sh (강제 3.10)
#  - pyenv 3.10.13만 허용
#  - 시스템 python3(=3.13)로는 절대 폴백하지 않음
# ===============================================

# 1) 후보 경로 지정
PYENV_PY="$HOME/.pyenv/versions/3.10.13/bin/python3"

# 2) 우선순위: pyenv 3.10.13
if [ -x "$PYENV_PY" ]; then
  python_cmd="$PYENV_PY"
else
  echo "[ERROR] Python 3.10.13가 발견되지 않았습니다."
  echo "        아래 명령어로 설치해 주세요:"
  echo "        - pyenv install 3.10.13"
  exit 1
fi

# 가시적 로그
echo "[INFO] Using python_cmd=$python_cmd"
"$python_cmd" -V || true

# 3) macOS MPS(Apple GPU) 폴백 허용
export PYTORCH_ENABLE_MPS_FALLBACK=1

# 4) Mark as already sourced to prevent double sourcing
export WEBUI_USER_SOURCED=1

# 5) 실행 옵션 (macOS 기본 안정 구성)
#  - --no-half: MPS에서 FP16 이슈 회피 (필요 없으면 제거 가능)
#  - --opt-sdp-attention: 메모리 효율 어텐션
#  - --api: API 사용
#  - --listen: 내부 네트워크 오픈.
export COMMANDLINE_ARGS="--listen --skip-torch-cuda-test"

# 6) webui.sh 실행
exec bash "$(dirname "$0")/webui.sh" "$@"

# ===============================================