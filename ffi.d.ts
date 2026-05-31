declare module "node:ffi" {
  type TypeName =
    | "void"
    | "pointer"
    | "buffer"
    | "arraybuffer"
    | "function"
    | "bool"
    | "char"
    | "string"
    | "float"
    | "double"
    | "int8"
    | "uint8"
    | "int16"
    | "uint16"
    | "int32"
    | "uint32"
    | "int64"
    | "uint64"
    | "float32"
    | "float64";

  interface FunctionSignature {
    parameters?: TypeName[];
    arguments?: TypeName[];
    result?: TypeName;
    return?: TypeName;
    returns?: TypeName;
  }

  type ForeignFunction = ((...args: any[]) => any) & { pointer: bigint };

  export class DynamicLibrary {
    constructor(path: string | null);
    getFunction(name: string, signature: FunctionSignature): ForeignFunction;
  }

  export const types: {
    VOID: "void";
    POINTER: "pointer";
    BUFFER: "buffer";
    ARRAY_BUFFER: "arraybuffer";
    FUNCTION: "function";
    BOOL: "bool";
    CHAR: "char";
    STRING: "string";
    FLOAT: "float";
    DOUBLE: "double";
    INT_8: "int8";
    UINT_8: "uint8";
    INT_16: "int16";
    UINT_16: "uint16";
    INT_32: "int32";
    UINT_32: "uint32";
    INT_64: "int64";
    UINT_64: "uint64";
    FLOAT_32: "float32";
    FLOAT_64: "float64";
  };

  export const suffix: 'dylib' | 'so' | 'dll';  
}
