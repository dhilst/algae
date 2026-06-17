# Generated from rbac.alg
from __future__ import annotations

from typing import NewType

UserId = NewType("UserId", str)
Role = NewType("Role", str)
Permission = NewType("Permission", str)


class RBAC:

    def __init__(self) -> None:
        self._users: set[UserId] = set()
        self._roles: set[Role] = set()
        self._permissions: set[Permission] = set()
        self._user_roles: dict[UserId, set[Role]] = {}
        self._role_perms: dict[Role, set[Permission]] = {}

    def _check_invariants(self) -> None:
        assert set(self._user_roles.keys()) <= self._users
        assert set(self._role_perms.keys()) <= self._roles
        for u in self._user_roles:
            assert self._user_roles[u] <= self._roles
        for r in self._role_perms:
            assert self._role_perms[r] <= self._permissions

    # --- Users ---

    def add_user(self, u: UserId) -> None:
        if u in self._users:
            raise ValueError(f"user {u!r} already exists")
        self._users.add(u)
        self._user_roles[u] = set()
        self._check_invariants()

    def remove_user(self, u: UserId) -> None:
        if u not in self._users:
            raise ValueError(f"user {u!r} does not exist")
        self._users.discard(u)
        self._user_roles.pop(u, None)
        self._check_invariants()

    # --- Roles ---

    def add_role(self, r: Role) -> None:
        if r in self._roles:
            raise ValueError(f"role {r!r} already exists")
        self._roles.add(r)
        self._role_perms[r] = set()
        self._check_invariants()

    def remove_role(self, r: Role) -> None:
        if r not in self._roles:
            raise ValueError(f"role {r!r} does not exist")
        self._roles.discard(r)
        self._role_perms.pop(r, None)
        for u in self._user_roles:
            self._user_roles[u].discard(r)
        self._check_invariants()

    # --- Permissions ---

    def add_permission(self, p: Permission) -> None:
        if p in self._permissions:
            raise ValueError(f"permission {p!r} already exists")
        self._permissions.add(p)
        self._check_invariants()

    def remove_permission(self, p: Permission) -> None:
        if p not in self._permissions:
            raise ValueError(f"permission {p!r} does not exist")
        self._permissions.discard(p)
        for r in self._role_perms:
            self._role_perms[r].discard(p)
        self._check_invariants()

    # --- Assignments ---

    def assign_role(self, u: UserId, r: Role) -> None:
        if u not in self._users:
            raise ValueError(f"user {u!r} does not exist")
        if r not in self._roles:
            raise ValueError(f"role {r!r} does not exist")
        self._user_roles[u].add(r)
        self._check_invariants()

    def revoke_role(self, u: UserId, r: Role) -> None:
        if u not in self._users:
            raise ValueError(f"user {u!r} does not exist")
        if r not in self._roles:
            raise ValueError(f"role {r!r} does not exist")
        self._user_roles[u].discard(r)
        self._check_invariants()

    def grant_permission(self, r: Role, p: Permission) -> None:
        if r not in self._roles:
            raise ValueError(f"role {r!r} does not exist")
        if p not in self._permissions:
            raise ValueError(f"permission {p!r} does not exist")
        self._role_perms[r].add(p)
        self._check_invariants()

    def revoke_permission(self, r: Role, p: Permission) -> None:
        if r not in self._roles:
            raise ValueError(f"role {r!r} does not exist")
        if p not in self._permissions:
            raise ValueError(f"permission {p!r} does not exist")
        self._role_perms[r].discard(p)
        self._check_invariants()

    # --- Queries ---

    def effective_perms(self, u: UserId) -> set[Permission]:
        if u not in self._users:
            raise ValueError(f"user {u!r} does not exist")
        result: set[Permission] = set()
        for r in self._user_roles.get(u, set()):
            result |= self._role_perms.get(r, set())
        return result

    def is_authorized(self, u: UserId, p: Permission) -> bool:
        if u not in self._users:
            raise ValueError(f"user {u!r} does not exist")
        return p in self.effective_perms(u)

    def get_roles(self, u: UserId) -> set[Role]:
        if u not in self._users:
            raise ValueError(f"user {u!r} does not exist")
        return set(self._user_roles.get(u, set()))

    def get_permissions(self, r: Role) -> set[Permission]:
        if r not in self._roles:
            raise ValueError(f"role {r!r} does not exist")
        return set(self._role_perms.get(r, set()))
