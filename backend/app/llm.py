import os
import time

import httpx

from app.observability import llm_duration_seconds, record_llm_usage

GROQ_API_URL = "https://api.groq.com/openai/v1/chat/completions"
GROQ_MODEL = "openai/gpt-oss-20b"


async def call_groq(messages: list[dict]) -> str:
    api_key = os.environ.get("GROQ_API_KEY")
    if not api_key:
        raise RuntimeError("GROQ_API_KEY is not set (see infra/.env.example)")

    start = time.perf_counter()
    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            GROQ_API_URL,
            headers={"Authorization": f"Bearer {api_key}"},
            json={"model": GROQ_MODEL, "messages": messages, "temperature": 0.2},
        )
        response.raise_for_status()
        data = response.json()

    llm_duration_seconds.observe(time.perf_counter() - start)

    usage = data.get("usage", {})
    record_llm_usage(
        prompt_tokens=usage.get("prompt_tokens", 0),
        completion_tokens=usage.get("completion_tokens", 0),
    )

    return data["choices"][0]["message"]["content"]