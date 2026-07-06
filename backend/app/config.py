from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    postgres_user: str = "cairn"
    postgres_password: str = "changeme"
    postgres_db: str = "cairn"
    postgres_host: str = "db"
    postgres_port: int = 5432

    @property
    def database_url(self) -> str:
        return (
            f"postgresql://{self.postgres_user}:{self.postgres_password}"
            f"@{self.postgres_host}:{self.postgres_port}/{self.postgres_db}"
        )

    class Config:
        env_file = ".env"


settings = Settings()