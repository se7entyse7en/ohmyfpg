__NAME__ = "ohmyfpg"
__VERSION__ = "0.0.0"
__DESCRIPTION__ = "Oh My Fast Postgres!"

from ohmyfpg import ohmyfpg


def py_sum_as_string(a: int, b: int):
    """Proxy to call Rust implmentation of `sum_as_string`."""
    return ohmyfpg.sum_as_string(a, b)
