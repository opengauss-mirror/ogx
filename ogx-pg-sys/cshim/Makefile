# Portions Copyright 2019-2021 ZomboDB, LLC.
# Portions Copyright 2021-2022 Technology Concepts & Design, Inc.
# <support@tcdi.com>
#
# All rights reserved.
#
# Use of this source code is governed by the MIT license that can be found in
# the LICENSE file.


MODULE_big = ogx-cshim-${PG_TARGET_VERSION}
OBJS = ogx-cshim.o $(WIN32RES)
PGFILEDESC = "ogx-cshim - C shim to help ogx"
STATIC_LIB_NAME = lib${MODULE_big}.a


${STATIC_LIB_NAME}: ogx-cshim.o
	ar crv $@ ogx-cshim.o

all: ${STATIC_LIB_NAME}

EXTRA_CLEAN += ${STATIC_LIB_NAME}

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)
