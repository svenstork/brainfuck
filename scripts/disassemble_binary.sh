#!/bin/bash
# Simple script to disassemble a binary dump of Arm64 code.

# We need to use the GNU version of objdump do that we can disassemble a binary file.
OBJDUMP_CMD="/opt/homebrew/opt/binutils/bin/objdump" 


# Check if a filename was provided
if [ -z "$1" ]; then
    echo "Usage: $0 <raw_binary_file>"
    echo "Requires a raw binary file to disassemble as AArch64."
    exit 1
fi

FILE="$1"

# Check if the file exists
if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found."
    exit 1
fi

echo "--- Disassembling '$FILE' as AArch64 Raw Binary ---"
echo ""

# The core command:
# -D: Disassemble all sections
# -b binary: Treat the input as raw binary format (no headers)
# -m aarch64: Specify the target architecture as ARM64
"$OBJDUMP_CMD" -D -b binary -m aarch64 "$FILE"
