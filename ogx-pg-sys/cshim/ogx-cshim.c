/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
#include "postgres.h"

#define IS_OG_3 (PG_VERSION_NUM >= 90000 && PG_VERSION_NUM < 10000)

#include "access/htup.h"
#include "access/htup_details.h"
#include "catalog/pg_type.h"
#include "nodes/pathnodes.h"
#include "nodes/pg_list.h"
#include "parser/parsetree.h"
#include "utils/memutils.h"
#include "utils/builtins.h"
#include "utils/array.h"
#include "storage/spin.h"


PGDLLEXPORT MemoryContext ogx_GetMemoryContextChunk(void *ptr);
MemoryContext ogx_GetMemoryContextChunk(void *ptr) {
    return GetMemoryChunkContext(ptr);
}

PGDLLEXPORT void ogx_elog(int32 level, char *message);
void ogx_elog(int32 level, char *message) {
    elog(level, "%s", message);
}

PGDLLEXPORT void ogx_elog_error(char *message);
void ogx_elog_error(char *message) {
    elog(ERROR, "%s", message);
}

PGDLLEXPORT void ogx_ereport(int level, int code, char *message, char *file, int lineno, int colno);
void ogx_ereport(int level, int code, char *message, char *file, int lineno, int colno) {
    ereport(level,
            (errcode(code),
                    errmsg("%s", message), errcontext_msg("%s:%d:%d", file, lineno, colno)));
}

PGDLLEXPORT void ogx_SET_VARSIZE(struct varlena *ptr, int size);
void ogx_SET_VARSIZE(struct varlena *ptr, int size) {
    SET_VARSIZE(ptr, size);
}

PGDLLEXPORT void ogx_SET_VARSIZE_SHORT(struct varlena *ptr, int size);
void ogx_SET_VARSIZE_SHORT(struct varlena *ptr, int size) {
    SET_VARSIZE_SHORT(ptr, size);
}

PGDLLEXPORT Datum ogx_heap_getattr(HeapTupleData *tuple, int attnum, TupleDesc tupdesc, bool *isnull);
Datum ogx_heap_getattr(HeapTupleData *tuple, int attnum, TupleDesc tupdesc, bool *isnull) {
    return heap_getattr(tuple, attnum, tupdesc, isnull);
}

PGDLLEXPORT TransactionId ogx_HeapTupleHeaderGetXmin(HeapTupleHeader htup_header);
TransactionId ogx_HeapTupleHeaderGetXmin(HeapTupleHeader htup_header) {
    return HeapTupleHeaderGetXmin(htup_header);
}

PGDLLEXPORT CommandId ogx_HeapTupleHeaderGetRawCommandId(HeapTupleHeader htup_header);
CommandId ogx_HeapTupleHeaderGetRawCommandId(HeapTupleHeader htup_header) {
    return HeapTupleHeaderGetRawCommandId(htup_header);
}

PGDLLEXPORT RangeTblEntry *ogx_planner_rt_fetch(Index index, PlannerInfo *plannerInfo);
RangeTblEntry *ogx_planner_rt_fetch(Index index, PlannerInfo *root) {
    return planner_rt_fetch(index, root);
}

PGDLLEXPORT void *ogx_list_nth(List *list, int nth);
void *ogx_list_nth(List *list, int nth) {
    return list_nth(list, nth);
}

PGDLLEXPORT int ogx_list_nth_int(List *list, int nth);
int ogx_list_nth_int(List *list, int nth) {
    return list_nth_int(list, nth);
}

PGDLLEXPORT Oid ogx_list_nth_oid(List *list, int nth);
Oid ogx_list_nth_oid(List *list, int nth) {
    return list_nth_oid(list, nth);
}

PGDLLEXPORT ListCell *ogx_list_nth_cell(List *list, int nth);
ListCell *ogx_list_nth_cell(List *list, int nth) {
    return list_nth_cell(list, nth);
}

PGDLLEXPORT Oid ogx_HeapTupleHeaderGetOid(HeapTupleHeader htup_header);
Oid ogx_HeapTupleHeaderGetOid(HeapTupleHeader htup_header) {
    return HeapTupleHeaderGetOid(htup_header);
}

PGDLLEXPORT char *ogx_GETSTRUCT(HeapTuple tuple);
char *ogx_GETSTRUCT(HeapTuple tuple) {
    return GETSTRUCT(tuple);
}

PGDLLEXPORT char *ogx_ARR_DATA_PTR(ArrayType *arr);
char *ogx_ARR_DATA_PTR(ArrayType *arr) {
    return ARR_DATA_PTR(arr);
}

PGDLLEXPORT int ogx_ARR_NELEMS(ArrayType *arr);
int ogx_ARR_NELEMS(ArrayType *arr) {
    return ArrayGetNItems(arr->ndim, ARR_DIMS(arr));
}

PGDLLEXPORT bits8 *ogx_ARR_NULLBITMAP(ArrayType *arr);
bits8 *ogx_ARR_NULLBITMAP(ArrayType *arr) {
    return ARR_NULLBITMAP(arr);
}

PGDLLEXPORT int ogx_ARR_NDIM(ArrayType *arr);
int ogx_ARR_NDIM(ArrayType *arr) {
    return ARR_NDIM(arr);
}

PGDLLEXPORT bool ogx_ARR_HASNULL(ArrayType *arr);
bool ogx_ARR_HASNULL(ArrayType *arr) {
    return ARR_HASNULL(arr);
}

PGDLLEXPORT int *ogx_ARR_DIMS(ArrayType *arr);
int *ogx_ARR_DIMS(ArrayType *arr){
    return ARR_DIMS(arr);
}

PGDLLEXPORT void ogx_SpinLockInit(volatile slock_t *lock);
void ogx_SpinLockInit(volatile slock_t *lock) {
    SpinLockInit(lock);
}

PGDLLEXPORT void ogx_SpinLockAcquire(volatile slock_t *lock);
void ogx_SpinLockAcquire(volatile slock_t *lock) {
    SpinLockAcquire(lock);
}

PGDLLEXPORT void ogx_SpinLockRelease(volatile slock_t *lock);
void ogx_SpinLockRelease(volatile slock_t *lock) {
    SpinLockRelease(lock);
}

PGDLLEXPORT bool ogx_SpinLockFree(slock_t *lock);
bool ogx_SpinLockFree(slock_t *lock) {
    return SpinLockFree(lock);
}
