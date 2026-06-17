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
# Convention: cast narrows the happy path of a fallible result.
op cast : (RBAC | Error) → RBAC;

var u : User;
var r : Role;
var p : Permission;

# A fully provisioned store: u exists, r exists and grants p, u holds r.
let with_user = empty_rbac().add_user(u);
let with_role = with_user.add_role(r);
let with_perm = with_role.grant_permission(r, p);
let with_assignment = with_perm.cast.assign_role(u, r);

# Happy path: full setup grants authorization.
axiom authorized_happy with_assignment.cast.authorized(u, p) = true;
# Unknown-user errors.
axiom assign_role_no_user empty_rbac().assign_role(u, r) = unknown_user;
axiom authorized_no_user empty_rbac().authorized(u, p) = unknown_user;
axiom remove_user_no_user empty_rbac().remove_user(u) = unknown_user;
axiom revoke_role_no_user empty_rbac().revoke_role(u, r) = unknown_user;
# Unknown-role errors.
axiom assign_role_no_role empty_rbac().add_user(u).assign_role(u, r) = unknown_role;
axiom grant_permission_no_role empty_rbac().grant_permission(r, p) = unknown_role;
axiom revoke_permission_no_role empty_rbac().revoke_permission(r, p) = unknown_role;
# A known user with no roles is denied.
axiom authorized_no_roles empty_rbac().add_user(u).authorized(u, p) = false;
# Revoking the role, the permission, or the user removes authorization.
axiom revoke_role_denies let revoked = with_assignment.cast.revoke_role(u, r) in
      revoked.cast.authorized(u, p) = false;
axiom revoke_permission_denies let revoked = with_assignment.cast.revoke_permission(r, p) in
      revoked.cast.authorized(u, p) = false;
axiom remove_user_denies let removed = with_assignment.cast.remove_user(u) in
      removed.cast.authorized(u, p) = unknown_user;
