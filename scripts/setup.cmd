echo "Make sure pipenv exists..."
WHERE poetry
IF %ERRORLEVEL% NEQ 0 python -m pip install pipenv

python3 -m pipenv install --dev