FROM python:slim-bookworm

RUN pip install poetry

WORKDIR /app/

COPY pyproject.toml ./
COPY poetry.lock ./
RUN poetry install --no-dev

COPY ./lithicrivers/ /app/lithicrivers/
# EXPOSE 5000

CMD ["poetry", "run", "python", "-m", "lithicrivers"]
