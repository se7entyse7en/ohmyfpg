__NAME__ = "ohmyfpg"
__VERSION__ = "0.1.0-dev.14"
__DESCRIPTION__ = "Oh My Fast Postgres!"

from ohmyfpg import ohmyfpg


async def connect(dsn: str) -> ohmyfpg.Connection:
    """Connect to the given `dsn`."""
    return await ohmyfpg.connect(dsn)
