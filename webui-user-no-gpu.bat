@echo off

REM C:\Users\[username]\AppData\Local\Programs\Python\Python310\python.exe
set PYTHON=C:\Users\kojeo\AppData\Local\Programs\Python\Python310\python.exe
set GIT=
set VENV_DIR=

REM cpu로만 이미지 생성하려면 아래 옵션을 전부 사용한다.
REM https://github.com/AUTOMATIC1111/stable-diffusion-webui/wiki/Command-Line-Arguments-and-Settings
set COMMANDLINE_ARGS=--use-cpu all --precision full --no-half --skip-torch-cuda-test

call webui.bat