# Braces were replaced by keyword blocks; `{` is no longer valid syntax
# (use `laws … qed`).
sort S : Sort;

theory T(
  S : Sort
) {
  law refl(x : S)
    |- x = x;
};
