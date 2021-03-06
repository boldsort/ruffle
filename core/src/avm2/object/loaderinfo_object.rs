//! Loader-info object

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::DisplayObject;
use crate::tag_utils::SwfMovie;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::Ref;
use std::sync::Arc;

/// Represents a thing which can be loaded by a loader.
#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub enum LoaderStream<'gc> {
    /// A loaded SWF movie.
    ///
    /// The associated `DisplayObject` is the root movieclip.
    Swf(Arc<SwfMovie>, DisplayObject<'gc>),
}

/// An Object which represents a loadable object, such as a SWF movie or image
/// resource.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct LoaderInfoObject<'gc>(GcCell<'gc, LoaderInfoObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct LoaderInfoObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The loaded stream that this gets it's info from.
    loaded_stream: Option<LoaderStream<'gc>>,
}

impl<'gc> LoaderInfoObject<'gc> {
    /// Box a movie into a loader info object.
    pub fn from_movie(
        movie: Arc<SwfMovie>,
        root: DisplayObject<'gc>,
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some(base_proto), ScriptObjectClass::NoClass);
        let loaded_stream = Some(LoaderStream::Swf(movie, root));

        Ok(LoaderInfoObject(GcCell::allocate(
            mc,
            LoaderInfoObjectData {
                base,
                loaded_stream,
            },
        ))
        .into())
    }

    /// Construct a loader-info subclass.
    pub fn derive(
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(
            Some(base_proto),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(LoaderInfoObject(GcCell::allocate(
            mc,
            LoaderInfoObjectData {
                base,
                loaded_stream: None,
            },
        ))
        .into())
    }
}

impl<'gc> TObject<'gc> for LoaderInfoObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn value_of(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        if let Some(class) = self.as_proto_class() {
            Ok(AvmString::new(mc, format!("[object {}]", class.read().name().local_name())).into())
        } else {
            Ok("[object Object]".into())
        }
    }

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::LoaderInfoObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(LoaderInfoObject(GcCell::allocate(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: None,
            },
        ))
        .into())
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::LoaderInfoObject(*self);
        let base = ScriptObjectData::base_new(
            Some(this),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(LoaderInfoObject(GcCell::allocate(
            activation.context.gc_context,
            LoaderInfoObjectData {
                base,
                loaded_stream: None,
            },
        ))
        .into())
    }

    /// Unwrap this object's loader stream
    fn as_loader_stream(&self) -> Option<Ref<LoaderStream<'gc>>> {
        if self.0.read().loaded_stream.is_some() {
            Some(Ref::map(self.0.read(), |v| {
                v.loaded_stream.as_ref().unwrap()
            }))
        } else {
            None
        }
    }
}
