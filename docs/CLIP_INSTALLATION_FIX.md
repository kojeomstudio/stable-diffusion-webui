# CLIP / open_clip 설치 오류 수정 내역 (CLIP Installation Fix)

## 이슈 개요 (Issue Overview)
- **에러 메시지**: `RuntimeError: Couldn't install clip.` 또는 `error: invalid command 'bdist_wheel'`
- **발생 지점**: `launch_utils.py`에서 `clip` 또는 `open_clip`을 설치하는 도중 발생.

## 원인 (Root Cause)
1. **`setuptools` 버전 호환성**: 최신 `setuptools` (v70.0.0 이상)에서 기존의 일부 메타데이터 처리 방식이 변경되면서, `pkg_resources`를 사용하는 오래된 패키지 빌드 시 오류가 발생할 수 있습니다.
2. **`bdist_wheel` 유틸리티 누락**: `--no-build-isolation` 플래그를 사용하여 패키지를 설치할 경우, 해당 환경(`venv`)에 `wheel` 패키지가 미리 설치되어 있어야 합니다. 그렇지 않으면 `wheel` 빌드 명령(`bdist_wheel`)을 수행할 수 없어 설치가 중단됩니다.

## 해결책 (Solution)
### 1. `launch_utils.py` 수정
`clip`과 `open_clip`을 설치하기 직전에, `setuptools` 버전을 70 미만으로 고정하고 `wheel` 패키지를 함께 설치하도록 강제하였습니다.
```python
if not is_installed("clip"):
    run_pip(f"install "setuptools<70.0.0" wheel", "setuptools and wheel for clip")
    run_pip(f"install {clip_package} --no-build-isolation", "clip")
```

### 2. `requirements_versions.txt` 수정
프로젝트 전체 환경에서 `wheel` 패키지를 기본적으로 사용할 수 있도록 `requirements_versions.txt`에 명시적으로 추가하였습니다.
```text
setuptools==69.5.1
wheel==0.43.0
```

## 검증 결과 (Verification)
- `venv` 폴더를 삭제한 후 `webui-user.bat`을 실행하여, `setuptools`와 `wheel`이 먼저 설치되고 이어서 `clip` 및 `open_clip`이 정상적으로 설치됨을 확인하였습니다.
- `--skip-torch-cuda-test` 환경에서도 정상적으로 라이브러리 의존성 체크가 완료되었습니다.
