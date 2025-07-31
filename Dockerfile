FROM python:3.12.0b3-slim

RUN pip install uv

WORKDIR /app/

COPY pyproject.toml ./
COPY README.md ./
COPY uv.lock ./
COPY ./lithicrivers/ /app/lithicrivers/
RUN uv sync --frozen

# EXPOSE 5000

CMD ["uv", "run", "python", "-m", "lithicrivers"]
