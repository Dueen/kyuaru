import neostandard from "neostandard";

export default neostandard({
  files: ["src/**/*.js", "tests/**/*.js"],
  ignores: ["dist/**/*"],
  noJsx: true,
  noStyle: true,
  semi: true,
});
