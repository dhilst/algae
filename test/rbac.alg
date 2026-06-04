sort User, Role, Permission, RBAC;
sort Error = {unknown_user, unknown_role};

op empty_rbac : -> RBAC;
op add_user : RBAC × User -> RBAC;
op add_role : RBAC × Role -> RBAC;
op assign_role : RBAC × User × Role -> RBAC | Error;
op grant_permission : RBAC × Role × Permission -> RBAC | Error;
op authorized : RBAC × User × Permission -> 𝔹 | Error;

var rbac : RBAC;
var u : User;
var r : Role;
var p : Permission;

axiom authorized(assign_role(grant_permission(add_role(add_user(rbac, u), r), r, p), u, r), u, p) = true;
axiom assign_role(empty_rbac(), u, r) = unknown_user;
