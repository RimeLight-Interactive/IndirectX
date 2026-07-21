#!/usr/bin/env python3
"""
update_hook_scaffolds.py  –  IndirectX hook scaffold generator

Run from the project root (next to src/).
Reads src/fn_typedefs/{device,context,swapchain}.rs and:
  1. Generates src/hooks/{device,context,swapchain}/<snake_name>.rs  (skips existing)
  2. Regenerates src/hooks/{device,context,swapchain}/mod.rs each run
"""

import os
import re
import sys
from pathlib import Path

# ── Paths ─────────────────────────────────────────────────────────────────────

SRC_DIR        = Path("src")
FN_TYPEDEFS    = SRC_DIR / "fn_typedefs"
HOOKS_DIR      = SRC_DIR / "hooks"

# ── VTable indices (hardcoded from parsed vtable structs) ─────────────────────
# IUnknown = 3 slots.  cfg-gated fields still occupy a slot (usize placeholder).
#
# ID3D11Device   base: IUnknown(3),          own fields start at 3
# ID3D11Context  base: ID3D11DeviceChild(7), own fields start at 7
# IDXGISwapChain base: IDXGIDeviceSubObject(8) [IUnknown(3)+IDXGIObject(4)+1], own at 8

VTABLE_INDICES: dict[str, int] = {
    # ── device (base: IUnknown = 3) ───────────────────────────────────────────
    "CreateBuffer":                              3,
    "CreateTexture1D":                           4,
    "CreateTexture2D":                           5,
    "CreateTexture3D":                           6,
    "CreateShaderResourceView":                  7,
    "CreateUnorderedAccessView":                 8,
    "CreateRenderTargetView":                    9,
    "CreateDepthStencilView":                   10,
    "CreateInputLayout":                        11,
    "CreateVertexShader":                       12,
    "CreateGeometryShader":                     13,
    "CreateGeometryShaderWithStreamOutput":     14,
    "CreatePixelShader":                        15,
    "CreateHullShader":                         16,
    "CreateDomainShader":                       17,
    "CreateComputeShader":                      18,
    "CreateClassLinkage":                       19,
    "CreateBlendState":                         20,
    "CreateDepthStencilState":                  21,
    "CreateRasterizerState":                    22,
    "CreateSamplerState":                       23,
    "CreateQuery":                              24,
    "CreatePredicate":                          25,
    "CreateCounter":                            26,
    "CreateDeferredContext":                    27,
    "OpenSharedResource":                       28,
    "CheckFormatSupport":                       29,
    "CheckMultisampleQualityLevels":            30,
    "CheckCounterInfo":                         31,
    "CheckCounter":                             32,
    "CheckFeatureSupport":                      33,
    "GetPrivateData":                           34,
    "SetPrivateData":                           35,
    "SetPrivateDataInterface":                  36,
    "GetFeatureLevel":                          37,
    "GetCreationFlags":                         38,
    "GetDeviceRemovedReason":                   39,
    "GetImmediateContext":                      40,
    "SetExceptionMode":                         41,
    "GetExceptionMode":                         42,

    # ── context (base: ID3D11DeviceChild = 7) ────────────────────────────────
    "VSSetConstantBuffers":                      7,
    "PSSetShaderResources":                      8,
    "PSSetShader":                               9,
    "PSSetSamplers":                            10,
    "VSSetShader":                              11,
    "DrawIndexed":                              12,
    "Draw":                                     13,
    "Map":                                      14,
    "Unmap":                                    15,
    "PSSetConstantBuffers":                     16,
    "IASetInputLayout":                         17,
    "IASetVertexBuffers":                       18,
    "IASetIndexBuffer":                         19,
    "DrawIndexedInstanced":                     20,
    "DrawInstanced":                            21,
    "GSSetConstantBuffers":                     22,
    "GSSetShader":                              23,
    "IASetPrimitiveTopology":                   24,
    "VSSetShaderResources":                     25,
    "VSSetSamplers":                            26,
    "Begin":                                    27,
    "End":                                      28,
    "GetData":                                  29,
    "SetPredication":                           30,
    "GSSetShaderResources":                     31,
    "GSSetSamplers":                            32,
    "OMSetRenderTargets":                       33,
    "OMSetRenderTargetsAndUnorderedAccessViews": 34,
    "OMSetBlendState":                          35,
    "OMSetDepthStencilState":                   36,
    "SOSetTargets":                             37,
    "DrawAuto":                                 38,
    "DrawIndexedInstancedIndirect":             39,
    "DrawInstancedIndirect":                    40,
    "Dispatch":                                 41,
    "DispatchIndirect":                         42,
    "RSSetState":                               43,
    "RSSetViewports":                           44,
    "RSSetScissorRects":                        45,
    "CopySubresourceRegion":                    46,
    "CopyResource":                             47,
    "UpdateSubresource":                        48,
    "CopyStructureCount":                       49,
    "ClearRenderTargetView":                    50,
    "ClearUnorderedAccessViewUint":             51,
    "ClearUnorderedAccessViewFloat":            52,
    "ClearDepthStencilView":                    53,
    "GenerateMips":                             54,
    "SetResourceMinLOD":                        55,
    "GetResourceMinLOD":                        56,
    "ResolveSubresource":                       57,
    "ExecuteCommandList":                       58,
    "HSSetShaderResources":                     59,
    "HSSetShader":                              60,
    "HSSetSamplers":                            61,
    "HSSetConstantBuffers":                     62,
    "DSSetShaderResources":                     63,
    "DSSetShader":                              64,
    "DSSetSamplers":                            65,
    "DSSetConstantBuffers":                     66,
    "CSSetShaderResources":                     67,
    "CSSetUnorderedAccessViews":                68,
    "CSSetShader":                              69,
    "CSSetSamplers":                            70,
    "CSSetConstantBuffers":                     71,
    "VSGetConstantBuffers":                     72,
    "PSGetShaderResources":                     73,
    "PSGetShader":                              74,
    "PSGetSamplers":                            75,
    "VSGetShader":                              76,
    "PSGetConstantBuffers":                     77,
    "IAGetInputLayout":                         78,
    "IAGetVertexBuffers":                       79,
    "IAGetIndexBuffer":                         80,
    "GSGetConstantBuffers":                     81,
    "GSGetShader":                              82,
    "IAGetPrimitiveTopology":                   83,
    "VSGetShaderResources":                     84,
    "VSGetSamplers":                            85,
    "GetPredication":                           86,
    "GSGetShaderResources":                     87,
    "GSGetSamplers":                            88,
    "OMGetRenderTargets":                       89,
    "OMGetRenderTargetsAndUnorderedAccessViews": 90,
    "OMGetBlendState":                          91,
    "OMGetDepthStencilState":                   92,
    "SOGetTargets":                             93,
    "RSGetState":                               94,
    "RSGetViewports":                           95,
    "RSGetScissorRects":                        96,
    "HSGetShaderResources":                     97,
    "HSGetShader":                              98,
    "HSGetSamplers":                            99,
    "HSGetConstantBuffers":                    100,
    "DSGetShaderResources":                    101,
    "DSGetShader":                             102,
    "DSGetSamplers":                           103,
    "DSGetConstantBuffers":                    104,
    "CSGetShaderResources":                    105,
    "CSGetUnorderedAccessViews":               106,
    "CSGetShader":                             107,
    "CSGetSamplers":                           108,
    "CSGetConstantBuffers":                    109,
    "ClearState":                              110,
    "Flush":                                   111,
    "GetType":                                 112,
    "GetContextFlags":                         113,
    "FinishCommandList":                       114,

    # ── swapchain (base: IDXGIDeviceSubObject = 8) ────────────────────────────
    # IUnknown(3) + IDXGIObject(4) + IDXGIDeviceSubObject(1) = 8
    "Present":                8,
    "GetBuffer":              9,
    "SetFullscreenState":    10,
    "GetFullscreenState":    11,
    "GetDesc":               12,
    "ResizeBuffers":         13,
    "ResizeFuffers":         13,   # typo in typedef — same slot, both keys safe
    "ResizeTarget":          14,
    "GetContainingOutput":   15,
    "GetFrameStatistics":    16,
    "GetLastPresentCount":   17,
}

def scrape_imports(typedef_path: Path) -> list[str]:
    """
    Pull every 'use ...' line from a fn_typedefs source file verbatim.
    Multi-line use blocks (e.g. use foo::{\\n    Bar,\\n}) are collapsed to one line.
    """
    text = typedef_path.read_text(encoding="utf-8")
    # Collapse multi-line use blocks into single lines first
    # A use block: starts with 'use', ends with ';', may span lines
    collapsed = re.sub(r'(use\s[^;]+;)', lambda m: re.sub(r'\s+', ' ', m.group(0)).strip(), text)
    lines = []
    for line in collapsed.splitlines():
        stripped = line.strip()
        if stripped.startswith("use ") and stripped.endswith(";"):
            lines.append(stripped)
    return lines

# ── Naming ────────────────────────────────────────────────────────────────────

RUST_KEYWORDS = {
    "as","box","break","const","continue","crate","else","enum","extern",
    "false","fn","for","if","impl","in","let","loop","match","mod","move",
    "mut","pub","ref","return","self","Self","static","struct","super",
    "trait","true","type","unsafe","use","where","while","dyn","abstract",
    "become","do","final","macro","override","priv","typeof","unsized",
    "virtual","yield","async","await","try",
}

# Prefixes to strip from D3D type names (tried longest-first)
STRIP_PREFIXES = ["ID3D11", "D3D11_", "ID3D", "D3D_", "D3D"]


def camel_to_snake(name: str) -> str:
    """CamelCase → lower_snake_case; digits act as word separators."""
    s = re.sub(r'([A-Z]+)([A-Z][a-z])', r'\1_\2', name)
    s = re.sub(r'([a-z])([A-Z])',        r'\1_\2', s)
    s = re.sub(r'([a-zA-Z])(\d)',        r'\1_\2', s)
    s = re.sub(r'(\d)([a-z])',            r'\1_\2', s)
    return s.lower()


def derive_param_name(raw_type: str, serial: list) -> str:
    """
    Given a Rust type string (pointer and all), return a parameter name.
    - First *mut c_void → handled by caller as 'this'
    - Types with D3D11_/ID3D11_/etc prefix → strip prefix, lowercase
    - Everything else (u32, usize, BOOL, DXGI_FORMAT …) → serial: a, b, c …
    """
    # strip pointer/const/mut noise, grab innermost named type
    inner = raw_type
    for tok in ("*mut", "*const", "Option<"):
        inner = inner.replace(tok, "")
    inner = inner.strip().rstrip(">")
    # last :: segment (handles windows_core::BOOL etc.)
    inner = inner.split("::")[-1].strip()

    for prefix in STRIP_PREFIXES:
        if inner.startswith(prefix):
            name = inner[len(prefix):].lower().lstrip("_")
            if not name:
                name = inner.lower()
            if name in RUST_KEYWORDS:
                name = name + "_val"
            return name

    # core / foreign type → serial
    ch = chr(ord('a') + serial[0])
    serial[0] += 1
    return ch


def build_params(raw_params: list[str]) -> list[tuple[str, str]]:
    """Returns [(name, type), …] for a function's parameter list."""
    result: list[tuple[str, str]] = []
    serial = [0]
    first = True

    for p in raw_params:
        p = p.strip()
        if not p:
            continue
        if first and p == "*mut c_void":
            result.append(("this", "*mut c_void"))
            first = False
            continue
        first = False

        name = derive_param_name(p, serial)

        # deduplicate
        taken = [r[0] for r in result]
        if name in taken:
            count = sum(1 for n in taken if n == name or re.match(rf'^{re.escape(name)}_\d+$', n))
            name = f"{name}_{count}"

        result.append((name, p))

    return result


# ── Typedef parser ────────────────────────────────────────────────────────────

def parse_typedefs(path: Path) -> list[tuple[str, list[str], str]]:
    """
    Returns [(TypeName, [param_types], return_str), …]
    return_str is e.g. "-> HRESULT" or "" for unit.
    """
    text = path.read_text(encoding="utf-8")
    pattern = re.compile(
        r'pub type (\w+)\s*=\s*unsafe extern "system" fn\s*\((.*?)\)\s*(->.*?)?;',
        re.DOTALL,
    )
    out = []
    for m in pattern.finditer(text):
        name   = m.group(1)
        params = [p.strip() for p in m.group(2).split(",") if p.strip()]
        ret    = (m.group(3) or "").strip()
        out.append((name, params, ret))
    return out


# ── Code generators ───────────────────────────────────────────────────────────

def gen_scaffold(typedef_name: str, params: list[str], ret: str, module: str, imports: list[str]) -> str:
    pairs = build_params(params)

    fn_params = "\n".join(f"    {name}: {typ}," for name, typ in pairs)
    call_args = ", ".join(name for name, _ in pairs)
    ret_str   = f" {ret}" if ret else ""

    imports = "\n".join(imports)

    return (
        f"use crate::fn_typedefs::{module}::{typedef_name};\n"
        f"use std::sync::OnceLock;\n"
        f"{imports}\n"
        f"\n"
        f"static ORIG_FUNC: OnceLock<{typedef_name}> = OnceLock::new();\n"
        f"\n"
        f"pub fn set_orig_func(func: usize) {{\n"
        f"    let _ = ORIG_FUNC.set(unsafe {{ std::mem::transmute(func) }});\n"
        f"}}\n"
        f"\n"
        f"pub fn hooked_func(\n"
        f"{fn_params}\n"
        f"){ret_str} {{\n"
        f"    unsafe {{\n"
        f"        let func = ORIG_FUNC.get().unwrap();\n"
        f"        func({call_args})\n"
        f"    }}\n"
        f"}}\n"
    )


def gen_mod_rs(module: str, typedef_names: list[str]) -> str:
    snake_names = [camel_to_snake(n) for n in typedef_names]

    mod_decls = "\n".join(f"pub mod {s};" for s in snake_names)

    entries = []
    for tname, sname in zip(typedef_names, snake_names):
        idx = VTABLE_INDICES.get(tname)
        if idx is not None:
            entries.append(f"        ({idx}, {sname}),")
        else:
            entries.append(f"        (/* TODO: vtable index for {tname} */ 0, {sname}),")
    hook_entries = "\n".join(entries)

    return (
        f"{mod_decls}\n"
        f"\n"
        f"use std::ffi::c_void;\n"
        f"use crate::make_hook_map;\n"
        f"\n"
        f"pub fn install_{module}_hooks(com: *mut c_void) {{\n"
        f"    let hook_map = make_hook_map!(\n"
        f"{hook_entries}\n"
        f"    );\n"
        f"    super::install_hooks(com, &hook_map);\n"
        f"}}\n"
    )


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    # Confirm we're in the right directory
    if not SRC_DIR.is_dir():
        sys.exit("ERROR: run this script from the project root (the directory containing src/)")

    for module in ("device", "context", "swapchain"):
        typedef_path = FN_TYPEDEFS / f"{module}.rs"
        hooks_subdir = HOOKS_DIR / module

        if not typedef_path.exists():
            print(f"[{module}] SKIP – {typedef_path} not found")
            continue

        typedefs = parse_typedefs(typedef_path)
        imports  = scrape_imports(typedef_path)
        print(f"\n[{module}] {len(typedefs)} typedef(s) | imports: {imports}")

        for typedef_name, raw_params, ret in typedefs:
            snake = camel_to_snake(typedef_name)
            out   = hooks_subdir / f"{snake}.rs"

            if out.exists():
                print(f"  SKIP  {out}  (already exists)")
                continue

            content = gen_scaffold(typedef_name, raw_params, ret, module, imports)
            out.write_text(content, encoding="utf-8")
            print(f"  GEN   {out}")

        # Regenerate mod.rs from ALL typedefs (including skipped/existing scaffolds)
        all_names = [t[0] for t in typedefs]
        mod_path  = hooks_subdir / "mod.rs"
        mod_path.write_text(gen_mod_rs(module, all_names), encoding="utf-8")
        print(f"  MOD   {mod_path}  (regenerated)")


if __name__ == "__main__":
    main()