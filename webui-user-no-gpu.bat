@echo off

REM C:\Users\[username]\AppData\Local\Programs\Python\Python310\python.exe
set PYTHON=C:\Users\kojeo\AppData\Local\Programs\Python\Python310\python.exe
set GIT=
set VENV_DIR=

REM --skip-torch-cuda-test (cuda 지원 GPU가 아닌 경우 사용)
set COMMANDLINE_ARGS=--skip-torch-cuda-test

call webui.bat