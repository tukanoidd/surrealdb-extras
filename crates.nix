{...}: {
  perSystem = {pkgs, ...}: {
    nci = {
      projects."surrealdb-extras" = {
        path = ./.;
        export = true;
      };
      crates = {
        surrealdb-extras = {};
        surrealdb-extras-proc-macro = {};
      };
    };
  };
}
