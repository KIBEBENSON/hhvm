/*
   +----------------------------------------------------------------------+
   | HipHop for PHP                                                       |
   +----------------------------------------------------------------------+
   | Copyright (c) 2010-present Facebook, Inc. (http://www.facebook.com)  |
   +----------------------------------------------------------------------+
   | This source file is subject to version 3.01 of the PHP license,      |
   | that is bundled with this package in the file LICENSE, and is        |
   | available through the world-wide-web at the following url:           |
   | http://www.php.net/license/3_01.txt                                  |
   | If you did not receive a copy of the PHP license and are unable to   |
   | obtain it through the world-wide-web, please send a note to          |
   | license@php.net so we can mail you a copy immediately.               |
   +----------------------------------------------------------------------+
*/
#ifndef incl_HPHP_CONTAINER_FUNCTIONS_H_
#define incl_HPHP_CONTAINER_FUNCTIONS_H_

#include "hphp/runtime/base/type-variant.h"
#include "hphp/runtime/base/collections.h"
#include "hphp/runtime/ext/collections/ext_collections.h"

namespace HPHP {

//////////////////////////////////////////////////////////////////////

inline bool isContainer(const Cell c) {
  assertx(cellIsPlausible(c));
  return isArrayLikeType(c.m_type) ||
         (c.m_type == KindOfObject && c.m_data.pobj->isCollection());
}

inline bool isContainer(const Variant& v) {
  return isContainer(*v.toCell());
}

inline bool isContainerOrNull(const Cell c) {
  assertx(cellIsPlausible(c));
  return isNullType(c.m_type) || isArrayLikeType(c.m_type) ||
         (c.m_type == KindOfObject && c.m_data.pobj->isCollection());
}

inline bool isContainerOrNull(const Variant& v) {
  return isContainerOrNull(*v.toCell());
}

inline bool isMutableContainer(const Cell c) {
  assertx(cellIsPlausible(c));
  return isArrayLikeType(c.m_type) ||
         (c.m_type == KindOfObject && c.m_data.pobj->isMutableCollection());
}

inline bool isMutableContainer(const Variant& v) {
  return isMutableContainer(*v.toCell());
}

inline size_t getContainerSize(const Cell c) {
  assertx(isContainer(c));
  if (isArrayLikeType(c.m_type)) {
    return c.m_data.parr->size();
  }
  assertx(c.m_type == KindOfObject && c.m_data.pobj->isCollection());
  return collections::getSize(c.m_data.pobj);
}

inline size_t getContainerSize(const Variant& v) {
  return getContainerSize(*v.toCell());
}

inline bool isPackedContainer(const Cell c) {
  assertx(isContainer(c));
  if (isArrayLikeType(c.m_type)) {
    return c.m_data.parr->hasPackedLayout();
  }

  return isVectorCollection(c.m_data.pobj->collectionType());
}

ALWAYS_INLINE
const Cell container_as_cell(const Variant& container) {
  const auto& cellContainer = *container.toCell();
  if (UNLIKELY(!isContainer(cellContainer))) {
    SystemLib::throwInvalidArgumentExceptionObject(
      "Parameter must be a container (array or collection)");
  }
  return cellContainer;
}

//////////////////////////////////////////////////////////////////////

/*
 * clsmeth compact container helpers.
 */
inline bool isClsMethCompactContainer(const Cell c) {
 return isContainer(c) || isClsMethType(c.m_type);
}

inline bool isClsMethCompactContainer(const Variant& v) {
 return isClsMethCompactContainer(*v.toCell());
}

inline size_t getClsMethCompactContainerSize(const Cell c) {
 return isClsMethType(c.m_type) ? 2 : getContainerSize(c);
}

inline size_t getClsMethCompactContainerSize(const Variant& v) {
 return getClsMethCompactContainerSize(*v.toCell());
}

inline Cell* castClsmethToContainerInplace(Cell* c) {
  if (isClsMethType(c->m_type)) {
    if (RuntimeOption::EvalHackArrDVArrs) {
      tvCastToVecInPlace(c);
    } else {
      tvCastToVArrayInPlace(c);
    }
  }
  return c;
}

//////////////////////////////////////////////////////////////////////
}

#endif
