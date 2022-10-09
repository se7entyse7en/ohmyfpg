__NAME__ = "ohmyfpg"
__VERSION__ = "0.4.0-dev.4"
__DESCRIPTION__ = "Oh My Fast Postgres!"

import sys
from typing import Dict

import numpy as np

from ohmyfpg import ohmyfpg


class Connection(object):
    """Wrapper connection object."""

    def __init__(self, obj):
        """Wrap the provided connection."""
        self._wrapped_obj = obj

    def __getattr__(self, attr):
        """Proxy everything to the wrappeed object."""
        if attr in self.__dict__:
            return getattr(self, attr)
        return getattr(self._wrapped_obj, attr)

    async def fetch(self, query_string: str) -> Dict[str, np.ndarray]:
        """Return the result of the query as `numpy` columns."""
        res = await self._wrapped_obj.fetch(query_string)
        d = {}
        for k, v in res.items():
            arr = np.frombuffer(v[0], dtype=np.dtype(v[1]))
            if sys.byteorder == 'little':
                arr = arr.byteswap().newbyteorder()

            d[k] = arr

        return d


async def connect(dsn: str) -> Connection:
    """Connect to the given `dsn`."""
    return Connection(await ohmyfpg.connect(dsn))
