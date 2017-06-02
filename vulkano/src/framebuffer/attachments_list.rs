// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::cmp;
use std::sync::Arc;
use SafeDeref;
use image::ImageViewAccess;
use image::sys::UnsafeImageView;
//use sync::AccessFlagBits;
//use sync::PipelineStages;

/// A list of attachments.
// TODO: rework this trait
pub unsafe trait AttachmentsList {
    /// Returns the image views of this list.
    // TODO: better return type
    fn raw_image_view_handles(&self) -> Vec<&UnsafeImageView>;

    /// Returns the dimensions of the intersection of the views. Returns `None` if the list is
    /// empty.
    ///
    /// For example if one view is 256x256x2 and another one is 128x512x3, then this function
    /// should return 128x256x2.
    fn intersection_dimensions(&self) -> Option<[u32; 3]>;

    // TODO: meh for API
    fn as_image_view_accesses(&self) -> Vec<&ImageViewAccess>;
}

unsafe impl<T> AttachmentsList for T where T: SafeDeref, T::Target: AttachmentsList {
    #[inline]
    fn raw_image_view_handles(&self) -> Vec<&UnsafeImageView> {
        (**self).raw_image_view_handles()
    }

    #[inline]
    fn intersection_dimensions(&self) -> Option<[u32; 3]> {
        (**self).intersection_dimensions()
    }

    #[inline]
    fn as_image_view_accesses(&self) -> Vec<&ImageViewAccess> {
        (**self).as_image_view_accesses()
    }
}

unsafe impl AttachmentsList for () {
    #[inline]
    fn raw_image_view_handles(&self) -> Vec<&UnsafeImageView> {
        vec![]
    }

    #[inline]
    fn intersection_dimensions(&self) -> Option<[u32; 3]> {
        None
    }

    #[inline]
    fn as_image_view_accesses(&self) -> Vec<&ImageViewAccess> {
        vec![]
    }
}

unsafe impl AttachmentsList for Vec<Arc<ImageViewAccess + Send + Sync>> {
    #[inline]
    fn raw_image_view_handles(&self) -> Vec<&UnsafeImageView> {
        self.iter().map(|img| img.inner()).collect()
    }

    #[inline]
    fn intersection_dimensions(&self) -> Option<[u32; 3]> {
        let mut dims = None;

        for view in self.iter() {
            debug_assert_eq!(view.dimensions().depth(), 1);

            match dims {
                None => {
                    dims = Some([
                        view.dimensions().width(),
                        view.dimensions().height(),
                        view.dimensions().array_layers()
                    ]);
                },
                Some(ref mut d) => {
                    d[0] = cmp::min(view.dimensions().width(), d[0]);
                    d[1] = cmp::min(view.dimensions().height(), d[1]);
                    d[2] = cmp::min(view.dimensions().array_layers(), d[2]);
                },
            }
        }

        dims
    }

    #[inline]
    fn as_image_view_accesses(&self) -> Vec<&ImageViewAccess> {
        self.iter().map(|p| &**p as &ImageViewAccess).collect()
    }
}

unsafe impl<A, B> AttachmentsList for (A, B)
    where A: AttachmentsList, B: ImageViewAccess
{
    #[inline]
    fn raw_image_view_handles(&self) -> Vec<&UnsafeImageView> {
        let mut list = self.0.raw_image_view_handles();
        list.push(self.1.inner());
        list
    }

    #[inline]
    fn intersection_dimensions(&self) -> Option<[u32; 3]> {
        let dims = {
            let d = self.1.dimensions();
            debug_assert_eq!(d.depth(), 1);
            [d.width(), d.height(), d.array_layers()]
        };

        let dims = match self.0.intersection_dimensions() {
            Some(d) => [
                cmp::min(d[0], dims[0]),
                cmp::min(d[1], dims[1]),
                cmp::min(d[2], dims[2])
            ],
            None => dims,
        };

        Some(dims)
    }

    #[inline]
    fn as_image_view_accesses(&self) -> Vec<&ImageViewAccess> {
        let mut list = self.0.as_image_view_accesses();
        list.push(&self.1);
        list
    }
}
