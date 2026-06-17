# Generated from kvstore.alg
from typing import TypeVar, Generic, Set

K = TypeVar("K")
V = TypeVar("V")


class KVStore(Generic[K, V]):

    def __init__(self) -> None:
        self._data: dict[K, V] = {}

    def put(self, k: K, v: V) -> None:
        self._data[k] = v

    def get(self, k: K) -> V:
        if k not in self._data:
            raise KeyError(k)
        return self._data[k]

    def delete(self, k: K) -> None:
        if k not in self._data:
            raise KeyError(k)
        del self._data[k]

    def has(self, k: K) -> bool:
        return k in self._data

    def size(self) -> int:
        return len(self._data)

    def keys(self) -> Set[K]:
        return set(self._data.keys())
