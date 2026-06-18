# Role-based access control: users, roles, permissions, and the fallible
# operations that provision and query them. Fallible operations return a happy
# value or an Error; the error-path equations pin down the failure modes.
sort User : Sort;
sort Role : Sort;
sort Permission : Sort;
sort RBAC : Sort;
sort Error : Sort;

op unknown_user : → Error;
op unknown_role : → Error;

op empty_rbac : → RBAC;
op add_user : RBAC × User → RBAC;
op add_role : RBAC × Role → RBAC;
op assign_role : RBAC × User × Role → RBAC | Error;
op grant_permission : RBAC × Role × Permission → RBAC | Error;
op authorized : RBAC × User × Permission → 𝔹 | Error;
op remove_user : RBAC × User → RBAC | Error;
op revoke_role : RBAC × User × Role → RBAC | Error;
op revoke_permission : RBAC × Role × Permission → RBAC | Error;

# Unknown-user errors.
eq assign_role_no_user(u : User, r : Role)
  empty_rbac().assign_role(u, r) = unknown_user;
eq authorized_no_user(u : User, p : Permission)
  empty_rbac().authorized(u, p) = unknown_user;
eq remove_user_no_user(u : User)
  empty_rbac().remove_user(u) = unknown_user;
eq revoke_role_no_user(u : User, r : Role)
  empty_rbac().revoke_role(u, r) = unknown_user;

# Unknown-role errors.
eq assign_role_no_role(u : User, r : Role)
  empty_rbac().add_user(u).assign_role(u, r) = unknown_role;
eq grant_permission_no_role(r : Role, p : Permission)
  empty_rbac().grant_permission(r, p) = unknown_role;
eq revoke_permission_no_role(r : Role, p : Permission)
  empty_rbac().revoke_permission(r, p) = unknown_role;

# A known user with no roles is denied.
eq authorized_no_roles(u : User, p : Permission)
  empty_rbac().add_user(u).authorized(u, p) = false;
