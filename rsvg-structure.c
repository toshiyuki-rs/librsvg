/* vim: set sw=4: -*- Mode: C; tab-width: 4; indent-tabs-mode: t; c-basic-offset: 4 -*- */
/*
   rsvg-structure.c: Rsvg's structual elements

   Copyright (C) 2000 Eazel, Inc.
   Copyright (C) 2002, 2003, 2004, 2005 Dom Lachowicz <cinamod@hotmail.com>
   Copyright (C) 2003, 2004, 2005 Caleb Moore <c.moore@student.unsw.edu.au>

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

   Authors: Raph Levien <raph@artofcode.com>, 
            Dom Lachowicz <cinamod@hotmail.com>, 
            Caleb Moore <c.moore@student.unsw.edu.au>
*/

#include "rsvg-structure.h"
#include "rsvg-image.h"
#include "rsvg-css.h"

#include <stdio.h>

void 
rsvg_defs_drawable_draw (RsvgDefsDrawable * self, RsvgDrawingCtx *ctx,
						 int dominate)
{
	RsvgState *state;

	state = &self->state;

	if (0 /*!state->visible*/)
		return;

	self->draw(self, ctx, dominate);
}

void
rsvg_start_g (RsvgHandle *ctx, RsvgPropertyBag *atts)
{
	RsvgState state;
	const char * klazz = NULL, * id = NULL, *value;
	rsvg_state_init(&state);

	if (rsvg_property_bag_size (atts))
		{
			if ((value = rsvg_property_bag_lookup (atts, "class")))
				klazz = value;
			if ((value = rsvg_property_bag_lookup (atts, "id")))
				id = value;

			rsvg_parse_style_attrs (ctx, &state, "g", klazz, id, atts);
		}	
  
	rsvg_push_def_group (ctx, id, &state);
}

void
rsvg_end_g (RsvgHandle *ctx)
{
	rsvg_pop_def_group (ctx);
}

static void 
rsvg_defs_drawable_group_draw (RsvgDefsDrawable * self, RsvgDrawingCtx *ctx, 
							  int dominate)
{
	RsvgDefsDrawableGroup *group = (RsvgDefsDrawableGroup*)self;
	guint i;

	rsvg_state_reinherit_top(ctx, &self->state, dominate);

	rsvg_push_discrete_layer (ctx);	

	for (i = 0; i < group->children->len; i++)
		{
			rsvg_state_push(ctx);
			rsvg_defs_drawable_draw (g_ptr_array_index(group->children, i), 
									 ctx, 0);
			rsvg_state_pop(ctx);
		}			

	rsvg_pop_discrete_layer (ctx);
}

static void 
rsvg_defs_drawable_group_free (RsvgDefVal *self)
{
	RsvgDefsDrawableGroup *z = (RsvgDefsDrawableGroup *)self;
	rsvg_state_finalize (&z->super.state);
	g_ptr_array_free(z->children, TRUE);
	g_free (z);
}

RsvgDefsDrawable * 
rsvg_push_def_group (RsvgHandle *ctx, const char * id, 
					 RsvgState *state)
{
	RsvgDefsDrawable * group;

	group = rsvg_push_part_def_group (ctx, id, state);

	if (group->parent != NULL)
		rsvg_defs_drawable_group_pack((RsvgDefsDrawableGroup *)group->parent, 
									  group);

	return group;
}

void
rsvg_pop_def_group (RsvgHandle *ctx)
{
	RsvgDefsDrawableGroup * group;

	group = (RsvgDefsDrawableGroup *)ctx->current_defs_group;
	if (group == NULL)
		return;
	ctx->current_defs_group = group->super.parent;

}

void 
rsvg_defs_drawable_group_pack (RsvgDefsDrawableGroup *self, RsvgDefsDrawable *child)
{
	if (self == NULL)
		return;
	g_ptr_array_add(self->children, child);
}

/* warning: takes ownership of @tempstate */
RsvgDefsDrawable * 
rsvg_push_part_def_group (RsvgHandle *ctx, const char * id, 
						  RsvgState * tempstate)
{
	RsvgDefsDrawableGroup *group;

	group = g_new (RsvgDefsDrawableGroup, 1);
	group->children = g_ptr_array_new();
	group->super.state = *tempstate;

	group->super.super.type = RSVG_DEF_PATH;
	group->super.super.free = rsvg_defs_drawable_group_free;
	group->super.draw = rsvg_defs_drawable_group_draw;

	rsvg_defs_set (ctx->defs, id, &group->super.super);

	group->super.parent = (RsvgDefsDrawable *)ctx->current_defs_group;

	ctx->current_defs_group = group;

	return &group->super;
}

static RsvgDefsDrawable *
rsvg_defs_drawable_use_resolve(RsvgDefsDrawableUse * self, RsvgDrawingCtx *ctx, double * affine_out)
{
	double affine[6];
	double x, y, width, height;
	x = self->x;
	y = self->y;
	width = self->w;
	height = self->h;

	RsvgDefVal * parent = self->link;

	if (parent != NULL)
		switch(parent->type)
			{
			case RSVG_DEF_PATH:
				{
					
					_rsvg_affine_translate(affine, x, y);
					_rsvg_affine_multiply(affine_out, affine, affine_out);	
					return (RsvgDefsDrawable *)parent;
				}
			case RSVG_DEF_SYMBOL:
				{
					RsvgDefsDrawable *drawable = 
						(RsvgDefsDrawable*)parent;
					RsvgDefsDrawableSymbol *symbol = 
						(RsvgDefsDrawableSymbol*)parent;
					
					if (symbol->has_vbox){
						rsvg_preserve_aspect_ratio
							(symbol->preserve_aspect_ratio, 
							 symbol->width, symbol->height, 
							 &width, &height, &x, &y);
						_rsvg_affine_translate(affine, x, y);
						_rsvg_affine_multiply(affine_out, affine, affine_out);	
						
						_rsvg_affine_scale(affine, width / symbol->width,
										 height / symbol->height);
						_rsvg_affine_multiply(affine_out, affine, affine_out);
						_rsvg_affine_translate(affine, -symbol->x, 
											 -symbol->y);
						_rsvg_affine_multiply(affine_out, affine, affine_out);
					}
					else {
						_rsvg_affine_translate(affine, x, y);
						_rsvg_affine_multiply(affine_out, affine, affine_out);	
					}
					
					return drawable;
				}
			default:
				break;
			}
	return NULL;
}

static void 
rsvg_defs_drawable_use_draw (RsvgDefsDrawable * self, RsvgDrawingCtx *ctx, 
							  int dominate)
{
	RsvgDefsDrawableUse *use = (RsvgDefsDrawableUse*)self;
	RsvgDefsDrawable * child;

	rsvg_state_reinherit_top(ctx,  &self->state, dominate);

	child = rsvg_defs_drawable_use_resolve(use, ctx, rsvg_state_current(ctx)->affine);

	/* If it can find nothing to draw... draw nothing */
	if (!use->link)
		return;

	rsvg_push_discrete_layer (ctx);

	rsvg_state_push(ctx);
	
	rsvg_defs_drawable_draw (child, ctx, 1);

	rsvg_state_pop(ctx);

	rsvg_pop_discrete_layer (ctx);
}	

static void 
rsvg_defs_drawable_use_free (RsvgDefVal *self)
{
	RsvgDefsDrawableUse *z = (RsvgDefsDrawableUse *)self;
	rsvg_state_finalize (&z->super.state);
	g_free (z);
}

static void
rsvg_defs_drawable_svg_draw (RsvgDefsDrawable * self, RsvgDrawingCtx *ctx, 
							 int dominate)
{
	RsvgDefsDrawableSvg * sself;
	RsvgState *state;
	gdouble affine[6];
	RsvgDefsDrawableGroup *group = (RsvgDefsDrawableGroup*)self;
	guint i;
	sself = (RsvgDefsDrawableSvg *)self;

	rsvg_state_reinherit_top(ctx, &self->state, dominate);

	rsvg_push_discrete_layer (ctx);

	if (!sself->overflow)
		rsvg_add_clipping_rect(ctx, sself->x, sself->y, sself->w, sself->h);

	state = rsvg_state_current (ctx);

	if (sself->has_vbox)
		{
			affine[0] = sself->w / sself->vbw;
			affine[1] = 0;
			affine[2] = 0;
			affine[3] = sself->h / sself->vbh;
			affine[4] = sself->x - sself->vbx * sself->w / sself->vbw;
			affine[5] = sself->y - sself->vby * sself->h / sself->vbh;
			_rsvg_affine_multiply(state->affine, affine, 
								  state->affine);
		}
	else
		{
			affine[0] = 1;
			affine[1] = 0;
			affine[2] = 0;
			affine[3] = 1;
			affine[4] = sself->x;
			affine[5] = sself->y;
			_rsvg_affine_multiply(state->affine, affine, 
								  state->affine);
		}

	for (i = 0; i < group->children->len; i++)
		{
			rsvg_state_push(ctx);

			rsvg_defs_drawable_draw (g_ptr_array_index(group->children, i), 
									 ctx, 0);
	
			rsvg_state_pop(ctx);
		}			

	rsvg_pop_discrete_layer (ctx);
}

static void 
rsvg_defs_drawable_svg_free (RsvgDefVal *self)
{
	RsvgDefsDrawableGroup *z = (RsvgDefsDrawableGroup *)self;
	rsvg_state_finalize (&z->super.state);
	g_ptr_array_free(z->children, TRUE);
	g_free (z);
}

void
rsvg_start_svg (RsvgHandle *ctx, RsvgPropertyBag *atts)
{
	int width = -1, height = -1, x = 0, y = 0;
	const char * id, *value;
	double vbox_x = 0, vbox_y = 0, vbox_w = 0, vbox_h = 0;
	gboolean has_vbox = FALSE, overflow = 0;
	id = NULL;
	RsvgDefsDrawableSvg * svg;
	RsvgDefsDrawableGroup * group;
	RsvgState state;
	rsvg_state_init(&state);

	if (rsvg_property_bag_size (atts))
		{
			if ((value = rsvg_property_bag_lookup (atts, "viewBox")))
				{
					has_vbox = rsvg_css_parse_vbox (value, &vbox_x, &vbox_y,
													&vbox_w, &vbox_h);
					/*we need to set width and height so we can use percentages for the size*/
					ctx->width = vbox_w;
					ctx->height = vbox_h;
				}
			if ((value = rsvg_property_bag_lookup (atts, "width")))
				width = rsvg_css_parse_normalized_length (value, ctx->dpi_x, ctx->width, 1);
			if ((value = rsvg_property_bag_lookup (atts, "height")))
				height = rsvg_css_parse_normalized_length (value, ctx->dpi_y, ctx->height, 1);
			if ((value = rsvg_property_bag_lookup (atts, "x")))
				x = rsvg_css_parse_normalized_length (value, ctx->dpi_x, ctx->width, 1);
			if ((value = rsvg_property_bag_lookup (atts, "y")))
				y = rsvg_css_parse_normalized_length (value, ctx->dpi_y, ctx->height, 1);
			if ((value = rsvg_property_bag_lookup (atts, "id")))
				id = value;
			if ((value = rsvg_property_bag_lookup (atts, "overflow")))
				overflow = rsvg_css_parse_overflow(value);
		}

	svg = g_new (RsvgDefsDrawableSvg, 1);
	group = &svg->super;
	svg->has_vbox = has_vbox;
	svg->preserve_aspect_ratio = RSVG_ASPECT_RATIO_XMID_YMID;

	svg->x = x; svg->y = y; svg->w = width; svg->h = height;
	svg->vbx = vbox_x; svg->vby = vbox_y; svg->vbw = vbox_w; svg->vbh = vbox_h;
	if (ctx->nest_level)
		svg->overflow = overflow;
	else
		svg->overflow = 1;
	
	if (has_vbox)
		{
			ctx->width = vbox_w;
			ctx->height = vbox_h;
		}
	else
		{	
			ctx->width = width;
			ctx->height = height;
		}

	group->children = g_ptr_array_new();
	group->super.state = state;

	group->super.super.type = RSVG_DEF_PATH;
	group->super.super.free = rsvg_defs_drawable_svg_free;
	group->super.draw = rsvg_defs_drawable_svg_draw;

	rsvg_defs_set (ctx->defs, id, &group->super.super);

	group->super.parent = (RsvgDefsDrawable *)ctx->current_defs_group;

	ctx->current_defs_group = group;

	if (group->super.parent != NULL)
		rsvg_defs_drawable_group_pack((RsvgDefsDrawableGroup *)group->super.parent, 
									  &group->super);

	if (!ctx->nest_level)
		ctx->treebase = group;
	ctx->nest_level++;
}

void
rsvg_end_svg(RsvgHandle *ctx)
{
	ctx->nest_level--;
	rsvg_pop_def_group (ctx);
}

void 
rsvg_start_use (RsvgHandle *ctx, RsvgPropertyBag *atts)
{
	const char * klazz = NULL, *id = NULL, *xlink_href = NULL, *value;
	double x = 0, y = 0, width = 0, height = 0, font_size;	
	gboolean got_width = FALSE, got_height = FALSE;
	RsvgState state;
	rsvg_state_init(&state);
	font_size = rsvg_state_current_font_size(ctx);

	if (rsvg_property_bag_size(atts))
		{
			if ((value = rsvg_property_bag_lookup (atts, "x")))
				x = rsvg_css_parse_normalized_length (value, ctx->dpi_x, (gdouble)ctx->width, font_size);
			if ((value = rsvg_property_bag_lookup (atts, "y")))
				y = rsvg_css_parse_normalized_length (value, ctx->dpi_y, (gdouble)ctx->height, font_size);
			if ((value = rsvg_property_bag_lookup (atts, "width"))) {
				width = rsvg_css_parse_normalized_length (value, ctx->dpi_x, (gdouble)ctx->height, font_size);
				got_width = TRUE;
			}
			if ((value = rsvg_property_bag_lookup (atts, "height"))) {
				height = rsvg_css_parse_normalized_length (value, ctx->dpi_y, (gdouble)ctx->height, font_size);
				got_height = TRUE;
			}
			if ((value = rsvg_property_bag_lookup (atts, "class")))
				klazz = value;
			if ((value = rsvg_property_bag_lookup (atts, "id")))
				id = value;
			if ((value = rsvg_property_bag_lookup (atts, "xlink:href")))
				xlink_href = value;
		}
	if (!xlink_href)
		return;

	rsvg_parse_style_attrs (ctx, &state, "use", klazz, id, atts);

	/* < 0 is an error, 0 disables rendering. TODO: handle positive values correctly */
	if (got_width || got_height)
		if (width <= 0. || height <= 0.)
			return;

	RsvgDefsDrawableUse * use;
	use = g_new (RsvgDefsDrawableUse, 1);
	use->super.state = state;
	use->super.super.type = RSVG_DEF_PATH;
	use->super.super.free = rsvg_defs_drawable_use_free;
	use->super.draw = rsvg_defs_drawable_use_draw;
	use->x = x;
	use->y = y;
	use->w = width;
	use->h = height;
	use->link = NULL;
	rsvg_defs_add_resolver (ctx->defs, &use->link, xlink_href);
	rsvg_defs_set (ctx->defs, id, &use->super.super);
	
	use->super.parent = (RsvgDefsDrawable *)ctx->current_defs_group;
	if (use->super.parent != NULL)
		rsvg_defs_drawable_group_pack((RsvgDefsDrawableGroup *)use->super.parent, 
									  &use->super);
}

static void 
rsvg_defs_drawable_symbol_free (RsvgDefVal *self)
{
	RsvgDefsDrawableGroup *z = (RsvgDefsDrawableGroup *)self;
	rsvg_state_finalize (&z->super.state);
	g_ptr_array_free(z->children, TRUE);
	g_free (z);
}

static void
rsvg_defs_drawable_symbol_draw (RsvgDefsDrawable * self, RsvgDrawingCtx *ctx, 
							 int dominate)
{
	RsvgDefsDrawableSymbol * sself;
	RsvgState *state;
	RsvgDefsDrawableGroup *group = (RsvgDefsDrawableGroup*)self;
	guint i;
	sself = (RsvgDefsDrawableSymbol *)self;

	rsvg_state_reinherit_top(ctx, &self->state, dominate);

	rsvg_push_discrete_layer (ctx);

	state = rsvg_state_current (ctx);

	if (!sself->overflow){
		rsvg_add_clipping_rect(ctx, sself->x, sself->y, sself->width, sself->height);
	}

	for (i = 0; i < group->children->len; i++)
		{
			rsvg_state_push(ctx);

			rsvg_defs_drawable_draw (g_ptr_array_index(group->children, i), 
									 ctx, 0);
	
			rsvg_state_pop(ctx);
		}			

	rsvg_pop_discrete_layer (ctx);
}


void 
rsvg_start_symbol(RsvgHandle *ctx, RsvgPropertyBag *atts)
{
	RsvgDefsDrawableSymbol *symbol;
	RsvgDefsDrawableGroup *group;
	RsvgState state;
	const char * klazz = NULL, *id = NULL, *value;

	rsvg_state_init(&state);
	symbol = g_new (RsvgDefsDrawableSymbol, 1);
	group = &symbol->super;
	symbol->has_vbox = 0;
	symbol->overflow = 0;
	symbol->preserve_aspect_ratio = RSVG_ASPECT_RATIO_XMID_YMID;

	if (rsvg_property_bag_size(atts))
		{
			if ((value = rsvg_property_bag_lookup (atts, "class")))
				klazz = value;
			if ((value = rsvg_property_bag_lookup (atts, "id")))
				id = value;
			if ((value = rsvg_property_bag_lookup (atts, "viewBox")))
				{
					symbol->has_vbox = rsvg_css_parse_vbox (value, 
															&symbol->x, 
															&symbol->y,
															&symbol->width, 
															&symbol->height);
					if (symbol->has_vbox)
						{
							ctx->width = symbol->width;
							ctx->height = symbol->height;
						}
				}
			if ((value = rsvg_property_bag_lookup (atts, "preserveAspectRatio")))
				symbol->preserve_aspect_ratio = rsvg_css_parse_aspect_ratio (value);			
			if ((value = rsvg_property_bag_lookup (atts, "overflow")))
				symbol->overflow = rsvg_css_parse_overflow(value);
		}

	rsvg_parse_style_attrs (ctx, &state, "symbol", klazz, id, atts);
	group->children = g_ptr_array_new();
	group->super.state = state;
	group->super.super.type = RSVG_DEF_SYMBOL;
	group->super.super.free = rsvg_defs_drawable_symbol_free;
	group->super.draw = rsvg_defs_drawable_symbol_draw;

	rsvg_defs_set (ctx->defs, id, &group->super.super);

	group->super.parent = (RsvgDefsDrawable *)ctx->current_defs_group;

	ctx->current_defs_group = group;
}

void
rsvg_start_defs (RsvgHandle *ctx, RsvgPropertyBag *atts)
{
	RsvgState state;
	const char * klazz = NULL, * id = NULL, *value;
	rsvg_state_init(&state);	

	if (rsvg_property_bag_size (atts))
		{
			if ((value = rsvg_property_bag_lookup (atts, "class")))
				klazz = value;
			if ((value = rsvg_property_bag_lookup (atts, "id")))
				id = value;

			rsvg_parse_style_attrs (ctx, &state, "defs", klazz, id, atts);
		}	
  
	/*I don't know if I am proud or discusted by this hack. It seems to 
	  have the same effect as the spec but not be in its spirit.*/
	rsvg_push_part_def_group (ctx, id, &state);
}

static void 
_rsvg_defs_drawable_switch_draw (RsvgDefsDrawable * self, RsvgDrawingCtx *ctx, 
								 int dominate)
{
	RsvgDefsDrawableGroup *group = (RsvgDefsDrawableGroup*)self;
	guint i;

	rsvg_state_reinherit_top(ctx, &self->state, dominate);

	rsvg_push_discrete_layer (ctx);	

	for (i = 0; i < group->children->len; i++)
		{
			RsvgDefsDrawable * drawable = g_ptr_array_index(group->children, i);

			if (drawable->state.cond_true) {
				rsvg_state_push(ctx);
				rsvg_defs_drawable_draw (g_ptr_array_index(group->children, i), 
										 ctx, 0);
				rsvg_state_pop(ctx);

				break; /* only render the 1st one */
			}
		}			

	rsvg_pop_discrete_layer (ctx);
}

void
rsvg_start_switch (RsvgHandle *ctx, RsvgPropertyBag *atts)
{
	RsvgState state;
	RsvgDefsDrawable * group;
	const char * klazz = NULL, * id = NULL, *value;

	rsvg_state_init(&state);
	
	if (rsvg_property_bag_size (atts))
		{
			if ((value = rsvg_property_bag_lookup (atts, "class")))
				klazz = value;
			if ((value = rsvg_property_bag_lookup (atts, "id")))
				id = value;

			rsvg_parse_style_attrs (ctx, &state, "switch", klazz, id, atts);
		}	

	group = rsvg_push_def_group (ctx, id, &state);
	group->draw = _rsvg_defs_drawable_switch_draw;
}

void
rsvg_end_switch (RsvgHandle *ctx)
{
	rsvg_pop_def_group (ctx);
}
