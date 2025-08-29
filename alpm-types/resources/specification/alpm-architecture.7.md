# NAME

architecture - architecture definition for ALPM based packages.

# DESCRIPTION

The **architecture** format is a custom string format that contains **CPU instruction set architectures**[1] and further identifiers and is used for representing the architecture of ALPM based packages.
This format is used in build scripts or file formats for package metadata (e.g. in **PKGBUILD**, **BUILDINFO**, **PKGINFO**) to describe the architecture of a package, or the architecture of dependency packages.

The **architecture** format comprises values defined by convention.
The values are custom keywords, most of which are derived directly from **CPU instruction set architectures**[1] or specific microarchitectures.

The **architecture** value must be covered by the set of alphanumeric characters and '_'.

# EXAMPLES

The below **architecture** values all relate to specific **CPU instruction set architectures**[1] or microarchitectures and can be used to specify a package architecture.
This usually means that packages of a specific **architecture** can only be used in a particular context.

```text
"any"
```

A package can be used on any hardware as its contents are not specific to any architecture.

```text
"x86_64"
```

A package can only be used on hardware compatible with the **x86-64**[2] instruction set version 1 and above (see **x86-64 microarchitecture levels**[3]).

```text
"x86_64_v2"
```

A package can only be used on hardware compatible with the **x86-64**[2] instruction set version 2 and above (see **x86-64 microarchitecture levels**[3]).

```text
"x86_64_v3"
```

A package can only be used on hardware compatible with the **x86-64**[2] instruction set version 3 and above (see **x86-64 microarchitecture levels**[3]).

```text
"x86_64_v4"
```

A package can only be used on hardware compatible with the **x86-64**[2] instruction set version 4 and above (see **x86-64 microarchitecture levels**[3]).

```text
"i686"
```

A package can only be used on hardware compatible with the **IA-32**[4] instruction set version 6 (aka. **P6**[5], or 'i686').
This architecture is considered legacy.

```text
"i486"
```

A package can only be used on hardware compatible with the **IA-32**[4] instruction set version 4 (aka. **i486**[6]).
This architecture is considered legacy.

```text
"pentium4"
```

A package can only be used on hardware compatible with the **Pentium 4**[7] microarchitecture.
This architecture is considered legacy.

```text
"armv7"
```

A package can only be used on hardware compatible with the **ARMv7 architecture family**[8].
This architecture is considered legacy.

```text
"armv8"
```

A package can only be used on hardware compatible with the **ARMv8 architecture family**[9].

```text
"aarch64"
```

A package can only be used on hardware compatible with the **AArch64**[10] (64-bit execution state of the **ARM architecture family**[11]).

```text
"riscv64"
```

A package can only be used on hardware compatible with the 64-bit variant of the **RISC-V**[12] instruction set architecture (**ISA**[13]).

```text
"loong64"
```

A package can only be used on hardware compatible with the 64-bit variant of the **LoongArch**[14] instruction set architecture (**ISA**[13]).

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **alpm-epoch**(7), **alpm-pkgrel**(7), **alpm-pkgver**(7), **vercmp**(8)

# NOTES

1. **CPU instruction set architectures**
   
   <https://en.wikipedia.org/wiki/Comparison_of_instruction_set_architectures#Instruction_sets>
1. **x86-64**
   
   <https://en.wikipedia.org/wiki/X86-64>
1. **x86-64 microarchitecture levels**
   
   <https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels>
1. **IA-32**
   
   <https://en.wikipedia.org/wiki/IA-32>
1. **P6**
   
   <https://en.wikipedia.org/wiki/P6_(microarchitecture)>
1. **i486**
   
   <https://en.wikipedia.org/wiki/I486>
1. **pentium4**
   
   <https://en.wikipedia.org/wiki/Pentium_4>
1. **ARMv7 architecture family**
   
   <https://en.wikipedia.org/wiki/ARM_architecture_family#32-bit_architecture>
1. **ARMv8 architecture family**
   
   <https://en.wikipedia.org/wiki/ARM_architecture_family#Armv8>
1. **AArch64**
   
   <https://en.wikipedia.org/wiki/AArch64>
1. **ARM architecture family**
   
   <https://en.wikipedia.org/wiki/ARM_architecture_family>
1. **RISC-V**
   
   <https://en.wikipedia.org/wiki/RISC-V>
1. **ISA**
   
   <https://en.wikipedia.org/wiki/Instruction_set_architecture>
1. **LoongArch**
   
   <https://en.wikipedia.org/wiki/Loongson#LoongArch>
