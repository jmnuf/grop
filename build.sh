
if ! hash rustc 2>/dev/null
then
    echo >&2 "[ERROR] Missing \`rustc\` compiler. This is a requirement, install it genius"
    exit 1
fi

if ! hash kinoko 2>/dev/null
then
   echo "[INFO] \`kinoko\` not found, defaulting to \`rustc\`"
   echo "[CMD] rustc src/main.rs -o build/grop"
   rustc src/main.rs -o build/grop
   if ! hash ./build/grop 2>/dev/null
   then
       exit 1
   fi
   ./build/grop -h
else
    echo "[CMD] kinoko build -r -- -h"
    kinoko build -r -- -h
fi
exit 0
