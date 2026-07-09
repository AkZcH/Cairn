FROM python:3.11-slim

WORKDIR /app

RUN pip install poetry --no-cache-dir

COPY pyproject.toml poetry.lock* ./
ENV POETRY_REQUESTS_TIMEOUT=120
RUN poetry config installer.max-workers 4 \
    && poetry config virtualenvs.create false \
    && poetry install --no-interaction --no-ansi --no-root

COPY app ./app

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]