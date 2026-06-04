sort User, Role, Permission, RBAC;
sort Error = {unknown_user, unknown_role};

op empty_rbac : → RBAC;
op add_user : RBAC × User → RBAC;
op add_role : RBAC × Role → RBAC;
op assign_role : RBAC × User × Role → RBAC | Error;
op grant_permission : RBAC × Role × Permission → RBAC | Error;
op authorized : RBAC × User × Permission → 𝔹 | Error;
op remove_user : RBAC × User → RBAC | Error;
op revoke_role : RBAC × User × Role → RBAC | Error;
op revoke_permission : RBAC × Role × Permission → RBAC | Error;

var rbac : RBAC;
var u : User;
var r : Role;
var p : Permission;

# Happy path: full setup grants authorization.
axiom let with_user = add_user(rbac, u) in let with_role = add_role(with_user, r) in let with_perm = grant_permission(with_role, r, p) in let with_assignment = assign_role(with_perm, u, r) in authorized(with_assignment, u, p) = true;
# Unknown-user errors.
axiom assign_role(empty_rbac(), u, r) = unknown_user;
axiom authorized(empty_rbac(), u, p) = unknown_user;
axiom remove_user(empty_rbac(), u) = unknown_user;
axiom revoke_role(empty_rbac(), u, r) = unknown_user;
# Unknown-role errors.
axiom assign_role(add_user(empty_rbac(), u), u, r) = unknown_role;
axiom grant_permission(empty_rbac(), r, p) = unknown_role;
axiom revoke_permission(empty_rbac(), r, p) = unknown_role;
# A known user with no roles is denied.
axiom authorized(add_user(empty_rbac(), u), u, p) = false;
# Revoking the role, the permission, or the user removes authorization.
axiom let with_user = add_user(empty_rbac(), u) in let with_role = add_role(with_user, r) in let with_perm = grant_permission(with_role, r, p) in let with_assignment = assign_role(with_perm, u, r) in let revoked = revoke_role(with_assignment, u, r) in authorized(revoked, u, p) = false;
axiom let with_user = add_user(empty_rbac(), u) in let with_role = add_role(with_user, r) in let with_perm = grant_permission(with_role, r, p) in let with_assignment = assign_role(with_perm, u, r) in let revoked = revoke_permission(with_assignment, r, p) in authorized(revoked, u, p) = false;
axiom let with_user = add_user(empty_rbac(), u) in let with_role = add_role(with_user, r) in let with_perm = grant_permission(with_role, r, p) in let with_assignment = assign_role(with_perm, u, r) in let removed = remove_user(with_assignment, u) in authorized(removed, u, p) = unknown_user;
