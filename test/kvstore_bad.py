# Generated from kvstore.alg — deliberately non-conforming
from typing import TypeVar, Generic

K = TypeVar("K")
V = TypeVar("V")


class KVStore(Generic[K, V]):

    def __init__(self) -> None:
        self._data: dict[K, V] = {}

    def put(self, k: K, v: V) -> None:
        self._data[k] = v

    def get(self, k: K) -> V:
        # BUG: no precondition check — returns None instead of raising
        return self._data.get(k)  # type: ignore

    # MISSING: delete operation

    def has(self, k: K) -> bool:
        return k in self._data

    def size(self) -> int:
        return len(self._data)

    # MISSING: keys operation
