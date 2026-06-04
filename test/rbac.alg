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

axiom let with_user = add_user(rbac, u) in
      let with_role = add_role(with_user, r) in
      let with_perm = grant_permission(with_role, r, p) in
      let with_assignment = assign_role(with_perm, u, r) in
      authorized(with_assignment, u, p) = true;
axiom assign_role(empty_rbac(), u, r) = unknown_user;
