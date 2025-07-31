echo "Installing uv..."
WHERE uv
IF %ERRORLEVEL% NEQ 0 python -m pip install uv

uv sync --extra dev