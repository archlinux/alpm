#!/bin/sh

# This script is a terrible hack that sets up some files under `/tmp` to run the test code in `main.rs`.
# TODO: replace with more reasonable testing infrastructure!

# Download some OpenPGP certificates for testing
ROOT1=/tmp/pki1
VDA_PATH=$ROOT1/vda/v1/arch/packages/default/openpgp

mkdir -p $VDA_PATH
cd $VDA_PATH || exit
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0x02fd1c7a934e614545849f19a6234074498e9cee" -O 02fd1c7a934e614545849f19a6234074498e9cee.openpgp
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0x717026a9d4779fc53940726640f557b731496106" -O 717026a9d4779fc53940726640f557b731496106.openpgp
wget "https://pgpkeys.eu/pks/lookup?op=get&search=0xBE2DBCF2B1E3E588AC325AEAA06B49470F8E620A" -O be2dbcf2b1e3e588ac325aeaa06b49470f8e620a.openpgp
cd - || exit

# Download a package and signature for testing
mkdir /tmp/arch
cd /tmp/arch || exit
wget https://ftp.agdsn.de/pub/mirrors/archlinux/core/os/x86_64/acl-2.3.2-1-x86_64.pkg.tar.zst
wget https://ftp.agdsn.de/pub/mirrors/archlinux/core/os/x86_64/acl-2.3.2-1-x86_64.pkg.tar.zst.sig
cd - || exit
