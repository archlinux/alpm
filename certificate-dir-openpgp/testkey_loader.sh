#!/bin/sh

ROOT1=/tmp/pki1

mkdir -p $ROOT1/vda/arch/packages/default/openpgp

cd $ROOT1/vda/arch/packages/default/openpgp || exit
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0x717026a9d4779fc53940726640f557b731496106" -O 0x717026a9d4779fc53940726640f557b731496106
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0xBE2DBCF2B1E3E588AC325AEAA06B49470F8E620A" -O 0xBE2DBCF2B1E3E588AC325AEAA06B49470F8E620A
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0xE240B57E2C4630BA768E2F26FC1B547C8D8172C8" -O 0xE240B57E2C4630BA768E2F26FC1B547C8D8172C8
cd - || exit
