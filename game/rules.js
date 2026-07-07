// Auto-generated map of standard-library rules/axioms to their source location,
// so the in-game help can link each imported rule to its definition on GitHub.
// Regenerate if the stdlib line numbers change.

const REPO = "https://github.com/dhilst/algae/blob/main/algae/stdlib/v1";

export const RULE_LOCATIONS = {
  "adt": {
    "fst_pair": 22,
    "pair_cases": 36,
    "product_reflect_intro": 46,
    "product_reflect_left": 55,
    "product_reflect_right": 63,
    "snd_pair": 29,
    "sum_cases": 77,
    "sum_reflect_elim": 104,
    "sum_reflect_left": 88,
    "sum_reflect_right": 96
  },
  "core": {
    "and_intro": 67,
    "and_left": 76,
    "and_right": 84,
    "biconditional_elim_left": 169,
    "biconditional_elim_right": 177,
    "biconditional_intro": 160,
    "exists_elim": 213,
    "exists_intro": 203,
    "false_elim": 152,
    "forall_elim": 185,
    "forall_intro": 194,
    "implication_elim": 126,
    "implication_intro": 118,
    "negation_elim": 143,
    "negation_intro": 135,
    "or_elim": 108,
    "or_intro_left": 92,
    "or_intro_right": 100,
    "refl": 6,
    "forward": 37,
    "backward": 26,
    "symmetry": 48,
    "transitivity": 57
  },
  "group": {},
  "list": {
    "append_associativity": 77,
    "append_cons_left": 31,
    "append_nil_left": 25,
    "append_nil_right": 71,
    "bind_append": 83,
    "bind_cons": 56,
    "bind_nil": 50,
    "bind_singleton": 64,
    "list_induction": 90,
    "return_def": 44,
    "singleton_def": 38
  },
  "monad": {},
  "nat": {
    "add_succ_left": 22,
    "add_zero_left": 17,
    "add_zero_right": 46,
    "induction": 37,
    "mul_succ_left": 32,
    "mul_zero_left": 27
  },
  "option": {
    "bind_none": 29,
    "bind_some": 35,
    "option_cases": 42,
    "return_def": 23
  },
  "result": {
    "bind_err": 33,
    "bind_ok": 26,
    "result_cases": 40,
    "return_def": 20
  }
};

// GitHub URL for a rule `name` imported from `module`, or null if unknown.
export function ruleUrl(module, name) {
  const m = RULE_LOCATIONS[module];
  if (!m || !(name in m)) return null;
  return `${REPO}/${module}.alg#L${m[name]}`;
}

