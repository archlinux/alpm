#!/bin/sh

ROOT1=/tmp/pki1

mkdir -p $ROOT1/vda/arch/packages/default/openpgp

cd $ROOT1/vda/arch/packages/default/openpgp || exit
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0x02fd1c7a934e614545849f19a6234074498e9cee" -O 02fd1c7a934e614545849f19a6234074498e9cee.openpgp
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0x717026a9d4779fc53940726640f557b731496106" -O 717026a9d4779fc53940726640f557b731496106.openpgp
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0xBE2DBCF2B1E3E588AC325AEAA06B49470F8E620A" -O be2dbcf2b1e3e588ac325aeaa06b49470f8e620a.openpgp
cd - || exit
