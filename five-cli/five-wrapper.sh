#!/bin/bash
# Five CLI Wrapper Script
# Works around global installation output issues

# Get the directory where this script is located
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Execute Five CLI with Node.js directly
exec node "$DIR/dist/index.js" "$@"