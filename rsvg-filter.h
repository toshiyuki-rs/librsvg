/* vim: set sw=4: -*- Mode: C; tab-width: 4; indent-tabs-mode: t; c-basic-offset: 4 -*- */
/*
   rsvg-filter.h : Provides filters

   Copyright (C) 2004 Caleb Moore

   This program is free software; you can redistribute it and/or
   modify it under the terms of the GNU Library General Public License as
   published by the Free Software Foundation; either version 2 of the
   License, or (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Library General Public License for more details.

   You should have received a copy of the GNU Library General Public
   License along with this program; if not, write to the
   Free Software Foundation, Inc., 59 Temple Place - Suite 330,
   Boston, MA 02111-1307, USA.

   Author: Caleb Moore <calebmm@tpg.com.au>
*/

#ifndef RSVG_FILTER_H
#define RSVG_FILTER_H

#include "rsvg.h"
#include "rsvg-defs.h"
#include <libxml/SAX.h>

G_BEGIN_DECLS

typedef struct _RsvgFilter RsvgFilter;

typedef enum {
	objectBoundingBox, userSpaceOnUse
} RsvgFilterUnits;

struct _RsvgFilter {
	RsvgDefVal super;
	int refcnt;
	GPtrArray * primitives;
	double x, y, width, height; 
	RsvgFilterUnits filterunits;
	RsvgFilterUnits primitiveunits;
};

void 
rsvg_filter_render (RsvgFilter *self, GdkPixbuf *source, GdkPixbuf *output, GdkPixbuf *bg, RsvgHandle *context);

void 
rsvg_start_filter (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_end_filter (RsvgHandle *ctx);

RsvgFilter *
rsvg_filter_parse (const RsvgDefs *defs, const char *str);

void 
rsvg_start_filter_primitive_blend (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_convolve_matrix (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_gaussian_blur (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_offset (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_merge (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_merge_node (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_colour_matrix (RsvgHandle *ctx, const xmlChar **atts);

void 
rsvg_start_filter_primitive_component_transfer (RsvgHandle * ctx,
												const xmlChar ** atts);

void 
rsvg_start_filter_primitive_component_transfer_function (RsvgHandle * ctx,
														 const xmlChar ** atts, char channel);

void
rsvg_start_filter_primitive_erode (RsvgHandle * ctx,
								   const xmlChar ** atts);

void
rsvg_start_filter_primitive_composite (RsvgHandle * ctx, const xmlChar ** atts);

void
rsvg_start_filter_primitive_flood (RsvgHandle * ctx,
								   const xmlChar ** atts);

void
rsvg_start_filter_primitive_displacement_map (RsvgHandle * ctx, const xmlChar ** atts);

void
rsvg_start_filter_primitive_turbulence (RsvgHandle * ctx, const xmlChar ** atts);

void
rsvg_start_filter_primitive_image (RsvgHandle * ctx, const xmlChar ** atts);

void
rsvg_start_filter_primitive_diffuse_lighting (RsvgHandle * ctx, const xmlChar ** atts);

void 
rsvg_start_filter_primitive_light_source (RsvgHandle * ctx,
										  const xmlChar ** atts, char type);

void
rsvg_start_filter_primitive_specular_lighting (RsvgHandle * ctx, const xmlChar ** atts);

void
rsvg_start_filter_primitive_tile (RsvgHandle * ctx, const xmlChar ** atts);

G_END_DECLS

#endif
