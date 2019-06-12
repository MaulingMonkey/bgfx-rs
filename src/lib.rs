// Copyright (c) 2015-2016, Johan Sköld.
// License: http://opensource.org/licenses/ISC

//! Rust wrapper around [bgfx].
//!
//! Before using this crate, ensure that you fullfill the build requirements for bgfx, as outlined
//! in its [documentation][bgfx building]. If you are compiling for an `msvc` target, make sure to
//! build this crate in a developer command prompt.
//!
//! ## Limitations
//!
//! - So far, only Windows, Linux, and OSX are supported.
//! - Far from all bgfx functionality is exposed. As more examples get ported, more functionality
//!   will be as well.
//!
//! *This API is still unstable, and very likely to change.*
//!
//! ## Basic Usage
//!
//! Before this crate can be used, some platform data must be initialized. See [`PlatformData`].
//!
//! ```should_panic
//! bgfx::PlatformData::new()
//!     .context(std::ptr::null_mut())
//!     .display(std::ptr::null_mut())
//!     .window(std::ptr::null_mut())
//!     .apply()
//!     .expect("Could not set platform data");
//! ```
//!
//! Once the platform data has been initialized, a new thread should be spawned to act as the main
//! thread. This thread should call [`bgfx::init`] or [`bgfx::Init::init`] to initialize bgfx. The
//! object returned by that function should be used to access bgfx API calls.
//!
//! ```no_run
//! std::thread::spawn(|| {
//!     let bgfx = bgfx::Init::default().init()
//!         .expect("Failed to initialize bgfx");
//!     // ...
//! });
//! ```
//!
//! Finally, the real main thread should act as the render thread, and repeatedly be calling
//! [`bgfx::render_frame`].
//!
//! ```no_run
//! loop {
//!     // This is probably also where you will want to pump the window event queue.
//!     bgfx::render_frame(-1);
//! }
//! ```
//!
//! See the examples for more in-depth usage.
//!
//! [bgfx]: https://github.com/bkaradzic/bgfx
//! [bgfx building]: https://bkaradzic.github.io/bgfx/build.html
//! [`bgfx::init`]: fn.init.html
//! [`bgfx::render_frame`]: fn.render_frame.html
//! [`PlatformData`]: struct.PlatformData.html

#[macro_use]
extern crate bgfx_sys;
#[macro_use]
extern crate bitflags;
extern crate libc;

use bgfx_sys::*;
use std::ffi;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

pub mod flags;

pub use flags::*;

/// Autoselect adapter.
pub const PCI_ID_NONE: u16 = BGFX_PCI_ID_NONE;

/// Software rasterizer.
pub const PCI_ID_SOFTWARE_RASTERIZER: u16 = BGFX_PCI_ID_SOFTWARE_RASTERIZER;

/// AMD adapter.
pub const PCI_ID_AMD: u16 = BGFX_PCI_ID_AMD;

/// Intel adapter.
pub const PCI_ID_INTEL: u16 = BGFX_PCI_ID_INTEL;

/// nVidia adapter.
pub const PCI_ID_NVIDIA: u16 = BGFX_PCI_ID_NVIDIA;

/// Renderer backend type.
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum RendererType {
    /// No rendering.
    Noop = BGFX_RENDERER_TYPE_NOOP,

    /// Direct3D 9.0.
    Direct3D9 = BGFX_RENDERER_TYPE_DIRECT3D9,

    /// Direct3D 11.0.
    Direct3D11 = BGFX_RENDERER_TYPE_DIRECT3D11,

    /// Direct3D 12.0.
    Direct3D12 = BGFX_RENDERER_TYPE_DIRECT3D12,

    /// GNM.
    GNM = BGFX_RENDERER_TYPE_GNM,

    /// Metal.
    Metal = BGFX_RENDERER_TYPE_METAL,

    /// OpenGLES.
    OpenGLES = BGFX_RENDERER_TYPE_OPENGLES,

    /// OpenGL.
    OpenGL = BGFX_RENDERER_TYPE_OPENGL,

    /// Vulkan.
    Vulkan = BGFX_RENDERER_TYPE_VULKAN,

    /// Use the most platform appropriate renderer.
    Default = BGFX_RENDERER_TYPE_COUNT,
}

/// `render_frame()` results.
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum RenderFrame {
    /// No context is available. This usually means the main thread has exited.
    NoContext = BGFX_RENDER_FRAME_NO_CONTEXT,

    /// The render was performed.
    Render = BGFX_RENDER_FRAME_RENDER,

    /// The render timed out.
    Timeout = BGFX_RENDER_FRAME_TIMEOUT,

    /// The renderer is exiting.
    Exiting = BGFX_RENDER_FRAME_EXITING,
}

/// Vertex attribute.
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Attrib {
    /// Position.
    Position = BGFX_ATTRIB_POSITION,

    /// Normal.
    Normal = BGFX_ATTRIB_NORMAL,

    /// Tangent.
    Tangent = BGFX_ATTRIB_TANGENT,

    /// Bitangent.
    Bitangent = BGFX_ATTRIB_BITANGENT,

    /// Color 0.
    Color0 = BGFX_ATTRIB_COLOR0,

    /// Color 1.
    Color1 = BGFX_ATTRIB_COLOR1,

    /// Index list.
    Indices = BGFX_ATTRIB_INDICES,

    /// Bone weight.
    Weight = BGFX_ATTRIB_WEIGHT,

    /// Texture coordinate 0.
    TexCoord0 = BGFX_ATTRIB_TEXCOORD0,

    /// Texture coordinate 1.
    TexCoord1 = BGFX_ATTRIB_TEXCOORD1,

    /// Texture coordinate 2.
    TexCoord2 = BGFX_ATTRIB_TEXCOORD2,

    /// Texture coordinate 3.
    TexCoord3 = BGFX_ATTRIB_TEXCOORD3,

    /// Texture coordinate 4.
    TexCoord4 = BGFX_ATTRIB_TEXCOORD4,

    /// Texture coordinate 5.
    TexCoord5 = BGFX_ATTRIB_TEXCOORD5,

    /// Texture coordinate 6.
    TexCoord6 = BGFX_ATTRIB_TEXCOORD6,

    /// Texture coordinate 7.
    TexCoord7 = BGFX_ATTRIB_TEXCOORD7,
}

/// Vertex attribute type.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum AttribType {
    /// Unsigned 8-bit integer.
    ///
    /// If the parameter is `true`, the value will be normalized between 0 and 1.
    Uint8(bool),

    /// Signed 8-bit integer.
    ///
    /// If the parameter is `true`, the value will be normalized between 0 and 1.
    Int8(bool),

    /// Unsigned 10-bit integer.
    ///
    /// If the parameter is `true`, the value will be normalized between 0 and 1.
    Uint10(bool),

    /// Signed 10-bit integer.
    ///
    /// If the parameter is `true`, the value will be normalized between 0 and 1.
    Int10(bool),

    /// Unsigned 16-bit integer.
    ///
    /// If the parameter is `true`, the value will be normalized between 0 and 1.
    Uint16(bool),

    /// Signed 16-bit integer.
    ///
    /// If the parameter is `true`, the value will be normalized between 0 and 1.
    Int16(bool),

    /// 16-bit float.
    Half,

    /// 32-bit float.
    Float,
}

/// Texture format type.
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum TextureFormat {
    // Compressed formats
    BC1         = BGFX_TEXTURE_FORMAT_BC1,                  /** ( 0) DXT1                           */
    BC2         = BGFX_TEXTURE_FORMAT_BC2,                  /** ( 1) DXT3                           */
    BC3         = BGFX_TEXTURE_FORMAT_BC3,                  /** ( 2) DXT5                           */
    BC4         = BGFX_TEXTURE_FORMAT_BC4,                  /** ( 3) LATC1/ATI1                     */
    BC5         = BGFX_TEXTURE_FORMAT_BC5,                  /** ( 4) LATC2/ATI2                     */
    BC6H        = BGFX_TEXTURE_FORMAT_BC6H,                 /** ( 5) BC6H                           */
    BC7         = BGFX_TEXTURE_FORMAT_BC7,                  /** ( 6) BC7                            */
    ETC1        = BGFX_TEXTURE_FORMAT_ETC1,                 /** ( 7) ETC1 RGB8                      */
    ETC2        = BGFX_TEXTURE_FORMAT_ETC2,                 /** ( 8) ETC2 RGB8                      */
    ETC2A       = BGFX_TEXTURE_FORMAT_ETC2A,                /** ( 9) ETC2 RGBA8                     */
    ETC2A1      = BGFX_TEXTURE_FORMAT_ETC2A1,               /** (10) ETC2 RGB8A1                    */
    PTC12       = BGFX_TEXTURE_FORMAT_PTC12,                /** (11) PVRTC1 RGB 2BPP                */
    PTC14       = BGFX_TEXTURE_FORMAT_PTC14,                /** (12) PVRTC1 RGB 4BPP                */
    PTC12A      = BGFX_TEXTURE_FORMAT_PTC12A,               /** (13) PVRTC1 RGBA 2BPP               */
    PTC14A      = BGFX_TEXTURE_FORMAT_PTC14A,               /** (14) PVRTC1 RGBA 4BPP               */
    PTC22       = BGFX_TEXTURE_FORMAT_PTC22,                /** (15) PVRTC2 RGBA 2BPP               */
    PTC24       = BGFX_TEXTURE_FORMAT_PTC24,                /** (16) PVRTC2 RGBA 4BPP               */
    ATC         = BGFX_TEXTURE_FORMAT_ATC,                  /** (17) ATC RGB 4BPP                   */
    ATCE        = BGFX_TEXTURE_FORMAT_ATCE,                 /** (18) ATCE RGBA 8 BPP explicit alpha */
    ATCI        = BGFX_TEXTURE_FORMAT_ATCI,                 /** (19) ATCI RGBA 8 BPP interpolated alpha */
    ASTC4x4     = BGFX_TEXTURE_FORMAT_ASTC4X4,              /** (20) ASTC 4x4 8.0 BPP               */
    ASTC5x5     = BGFX_TEXTURE_FORMAT_ASTC5X5,              /** (21) ASTC 5x5 5.12 BPP              */
    ASTC6x6     = BGFX_TEXTURE_FORMAT_ASTC6X6,              /** (22) ASTC 6x6 3.56 BPP              */
    ASTC8x5     = BGFX_TEXTURE_FORMAT_ASTC8X5,              /** (23) ASTC 8x5 3.20 BPP              */
    ASTC8x6     = BGFX_TEXTURE_FORMAT_ASTC8X6,              /** (24) ASTC 8x6 2.67 BPP              */
    ASTC10x5    = BGFX_TEXTURE_FORMAT_ASTC10X5,             /** (25) ASTC 10x5 2.56 BPP             */

    // Uncompressed formats
    Unknown     = BGFX_TEXTURE_FORMAT_UNKNOWN,              /** (26) Compressed formats above.      */
    R1          = BGFX_TEXTURE_FORMAT_R1,
    A8          = BGFX_TEXTURE_FORMAT_A8,
    R8          = BGFX_TEXTURE_FORMAT_R8,
    R8I         = BGFX_TEXTURE_FORMAT_R8I,
    R8U         = BGFX_TEXTURE_FORMAT_R8U,
    R8S         = BGFX_TEXTURE_FORMAT_R8S,
    R16         = BGFX_TEXTURE_FORMAT_R16,
    R16I        = BGFX_TEXTURE_FORMAT_R16I,
    R16U        = BGFX_TEXTURE_FORMAT_R16U,
    R16F        = BGFX_TEXTURE_FORMAT_R16F,
    R16S        = BGFX_TEXTURE_FORMAT_R16S,
    R32I        = BGFX_TEXTURE_FORMAT_R32I,
    R32U        = BGFX_TEXTURE_FORMAT_R32U,
    R32F        = BGFX_TEXTURE_FORMAT_R32F,
    RG8         = BGFX_TEXTURE_FORMAT_RG8,
    RG8I        = BGFX_TEXTURE_FORMAT_RG8I,
    RG8U        = BGFX_TEXTURE_FORMAT_RG8U,
    RG8S        = BGFX_TEXTURE_FORMAT_RG8S,
    RG16        = BGFX_TEXTURE_FORMAT_RG16,
    RG16I       = BGFX_TEXTURE_FORMAT_RG16I,
    RG16U       = BGFX_TEXTURE_FORMAT_RG16U,
    RG16F       = BGFX_TEXTURE_FORMAT_RG16F,
    RG16S       = BGFX_TEXTURE_FORMAT_RG16S,
    RG32I       = BGFX_TEXTURE_FORMAT_RG32I,
    RG32U       = BGFX_TEXTURE_FORMAT_RG32U,
    RG32F       = BGFX_TEXTURE_FORMAT_RG32F,
    RGB8        = BGFX_TEXTURE_FORMAT_RGB8,
    RGB8I       = BGFX_TEXTURE_FORMAT_RGB8I,
    RGB8U       = BGFX_TEXTURE_FORMAT_RGB8U,
    RGB8S       = BGFX_TEXTURE_FORMAT_RGB8S,
    RGB9E5F     = BGFX_TEXTURE_FORMAT_RGB9E5F,
    BGRA8       = BGFX_TEXTURE_FORMAT_BGRA8,
    RGBA8       = BGFX_TEXTURE_FORMAT_RGBA8,
    RGBA8I      = BGFX_TEXTURE_FORMAT_RGBA8I,
    RGBA8U      = BGFX_TEXTURE_FORMAT_RGBA8U,
    RGBA8S      = BGFX_TEXTURE_FORMAT_RGBA8S,
    RGBA16      = BGFX_TEXTURE_FORMAT_RGBA16,
    RGBA16I     = BGFX_TEXTURE_FORMAT_RGBA16I,
    RGBA16U     = BGFX_TEXTURE_FORMAT_RGBA16U,
    RGBA16F     = BGFX_TEXTURE_FORMAT_RGBA16F,
    RGBA16S     = BGFX_TEXTURE_FORMAT_RGBA16S,
    RGBA32I     = BGFX_TEXTURE_FORMAT_RGBA32I,
    RGBA32U     = BGFX_TEXTURE_FORMAT_RGBA32U,
    RGBA32F     = BGFX_TEXTURE_FORMAT_RGBA32F,
    R5G6B5      = BGFX_TEXTURE_FORMAT_R5G6B5,
    RGBA4       = BGFX_TEXTURE_FORMAT_RGBA4,
    RGB5A1      = BGFX_TEXTURE_FORMAT_RGB5A1,
    RGB10A2     = BGFX_TEXTURE_FORMAT_RGB10A2,
    RG11B10F    = BGFX_TEXTURE_FORMAT_RG11B10F,

    // Depth formats
    UnknownDepth= BGFX_TEXTURE_FORMAT_UNKNOWNDEPTH,
    D16         = BGFX_TEXTURE_FORMAT_D16,
    D24         = BGFX_TEXTURE_FORMAT_D24,
    D24S8       = BGFX_TEXTURE_FORMAT_D24S8,
    D32         = BGFX_TEXTURE_FORMAT_D32,
    D16F        = BGFX_TEXTURE_FORMAT_D16F,
    D24F        = BGFX_TEXTURE_FORMAT_D24F,
    D32F        = BGFX_TEXTURE_FORMAT_D32F,
    D0S8        = BGFX_TEXTURE_FORMAT_D0S8,
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Resolution(bgfx_resolution_t);

impl Resolution {
    pub fn with_format(mut self, format: TextureFormat) -> Self { self.0.format = format as i32; self }
    pub fn with_width(mut self, width: u32) -> Self { self.0.width = width; self }
    pub fn with_height(mut self, height: u32) -> Self { self.0.height = height; self }
}

impl Default for Resolution {
    fn default() -> Self {
        Self(bgfx_resolution_t {
            // XXX?
            format:             TextureFormat::Unknown as i32 as bgfx_texture_format,
            width:              0,
            height:             0,
            reset:              0,
            numBackBuffers:     0,
            maxFrameLatency:    0,
        })
    }
}

pub type ViewId = bgfx_view_id_t;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct InitLimits(bgfx_init_limits_t);

impl Default for InitLimits {
    fn default() -> Self {
        Self(bgfx_init_limits_t{
            maxEncoders:        0,
            transientVbSize:    0,
            transientIbSize:    0,
        })
    }
}

/// bgfx error.
#[derive(Debug)]
pub enum BgfxError {
    /// An invalid display was provided in the platform data.
    InvalidDisplay,

    /// An invalid window was provided in the platform data.
    InvalidWindow,

    /// Initialization failed.
    InitFailed,
}

/// bgfx-managed buffer of memory.
///
/// It can be created by either copying existing data through [`copy(...)`], or by referencing
/// existing memory directly through [`reference(...)`].
///
/// [`copy(...)`]: #method.copy
/// [`reference(...)`]: #method.reference
pub struct Memory<'b> {
    handle: *const bgfx_memory_t,
    _phantom: PhantomData<&'b ()>,
}

impl<'b> Memory<'b> {

    /// Copies the source data into a new bgfx-managed buffer.
    ///
    /// **IMPORTANT:** If this buffer is never passed into a bgfx call, the memory will never be
    /// freed, and will leak.
    #[inline]
    pub fn copy<'d, T>(_bgfx: &'b Bgfx, data: &'d [T]) -> Memory<'b> {
        unsafe {
            let handle = bgfx_copy(data.as_ptr() as *const std::os::raw::c_void,
                                             mem::size_of_val(data) as u32);
            Memory { handle: handle, _phantom: PhantomData }
        }
    }

    /// Creates a reference to the source data for passing into bgfx. When using this constructor
    /// over the `copy` call, no copy will be created. bgfx will read the source memory directly.
    ///
    /// *Note that this is only valid for memory that will live for longer than the bgfx object,
    /// as it's the only way we can guarantee that the memory will still be valid until bgfx has
    /// finished using it.*
    #[inline]
    pub fn reference<T>(_bgfx: &'b Bgfx, data: &'b [T]) -> Memory<'b> {
        // TODO: The lifetime setup probably isn't 100% complete. Need to figure out how we can
        // guarantee that `data` will outlast `_bgfx`.
        unsafe {
            let handle = bgfx_make_ref(data.as_ptr() as *const std::os::raw::c_void,
                                                 mem::size_of_val(data) as u32);
            Memory { handle: handle, _phantom: PhantomData }
        }
    }

}

/// Shader program.
///
/// The program holds a vertex shader and a fragment shader.
pub struct Program<'s> {
    handle: bgfx_program_handle_t,
    _vsh: Shader<'s>,
    _fsh: Shader<'s>,
}

impl<'s> Program<'s> {

    /// Creates a new program from a vertex shader and a fragment shader. Ownerships of the shaders
    /// are moved to the program.
    #[inline]
    pub fn new(vsh: Shader<'s>, fsh: Shader<'s>) -> Program<'s> {
        unsafe {
            let handle = bgfx_create_program(vsh.handle, fsh.handle, false);
            Program { handle: handle, _vsh: vsh, _fsh: fsh }
        }
    }

}

impl<'s> Drop for Program<'s> {

    #[inline]
    fn drop(&mut self) {
        unsafe { bgfx_destroy_program(self.handle) }
    }

}

/// Shader.
pub struct Shader<'m> {
    handle: bgfx_shader_handle_t,
    _phantom: PhantomData<&'m ()>,
}

impl<'m> Shader<'m> {

    /// Creates a new shader from bgfx-managed memory.
    #[inline]
    pub fn new(data: Memory<'m>) -> Shader<'m> {
        unsafe {
            let handle = bgfx_create_shader(data.handle);
            Shader { handle: handle, _phantom: PhantomData }
        }
    }

}

impl<'m> Drop for Shader<'m> {

    #[inline]
    fn drop(&mut self) {
        unsafe { bgfx_destroy_shader(self.handle) }
    }

}

/// Vertex index buffer.
pub struct IndexBuffer<'m> {
    handle: bgfx_index_buffer_handle_t,
    _phantom: PhantomData<&'m ()>,
}

impl<'m> IndexBuffer<'m> {

    /// Creates a new index buffer from bgfx-managed memory.
    #[inline]
    pub fn new(indices: Memory<'m>, flags: BufferFlags) -> IndexBuffer<'m> {
        unsafe {
            let handle = bgfx_create_index_buffer(indices.handle, flags.bits());
            IndexBuffer { handle: handle, _phantom: PhantomData }
        }
    }

}

impl<'m> Drop for IndexBuffer<'m> {

    #[inline]
    fn drop(&mut self) {
        unsafe { bgfx_destroy_index_buffer(self.handle) }
    }

}

/// Vertex data buffer.
pub struct VertexBuffer<'m> {
    handle: bgfx_vertex_buffer_handle_t,
    _phantom: PhantomData<&'m Bgfx>,
}

impl<'m> VertexBuffer<'m> {

    /// Creates a new vertex buffer from bgfx-managed memory.
    #[inline]
    pub fn new<'v>(verts: Memory<'m>,
                   decl: &'v VertexDecl,
                   flags: BufferFlags)
                   -> VertexBuffer<'m> {
        unsafe {
            let handle = bgfx_create_vertex_buffer(verts.handle,
                                                             &decl.decl,
                                                             flags.bits());
            VertexBuffer { handle: handle, _phantom: PhantomData }
        }
    }

}

impl<'m> Drop for VertexBuffer<'m> {

    #[inline]
    fn drop(&mut self) {
        unsafe { bgfx_destroy_vertex_buffer(self.handle) }
    }

}

/// Describes the structure of a vertex.
pub struct VertexDecl {
    decl: bgfx_vertex_decl_t,
}

impl VertexDecl {

    /// Creates a new vertex declaration using a [`VertexDeclBuilder`].
    ///
    /// # Example
    ///
    /// ```
    /// bgfx::VertexDecl::new(None)
    ///                  .add(bgfx::Attrib::Position, 3, bgfx::AttribType::Float)
    ///                  .add(bgfx::Attrib::Color0, 4, bgfx::AttribType::Uint8(true))
    ///                  .end();
    /// ```
    ///
    /// [`VertexDeclBuilder`]: struct.VertexDeclBuilder.html
    #[inline]
    pub fn new(renderer: Option<RendererType>) -> VertexDeclBuilder {
        unsafe {
            let renderer = mem::transmute(renderer.unwrap_or(RendererType::Noop));
            let mut descr = VertexDeclBuilder { decl: mem::uninitialized() };
            bgfx_vertex_decl_begin(&mut descr.decl, renderer);
            descr
        }
    }

}

/// Builder for `VertexDecl` instances.
pub struct VertexDeclBuilder {
    decl: bgfx_vertex_decl_t,
}

impl VertexDeclBuilder {

    /// Adds a field to the structure descriptor. See [`VertexDecl::new`] for an example.
    ///
    /// [`VertexDecl::new`]: struct.VertexDecl.html#method.new
    pub fn add(&mut self, attrib: Attrib, count: u8, kind: AttribType) -> &mut Self {
        let mut normalized = false;
        let mut as_int = false;

        let kind = match kind {
            AttribType::Uint8(n) => {
                normalized = n;
                BGFX_ATTRIB_TYPE_UINT8
            }
            AttribType::Int8(n) => {
                normalized = n;
                as_int = true;
                BGFX_ATTRIB_TYPE_UINT8
            }
            AttribType::Uint10(n) => {
                normalized = n;
                BGFX_ATTRIB_TYPE_UINT10
            }
            AttribType::Int10(n) => {
                normalized = n;
                as_int = true;
                BGFX_ATTRIB_TYPE_UINT10
            }
            AttribType::Uint16(n) => {
                normalized = n;
                BGFX_ATTRIB_TYPE_INT16
            }
            AttribType::Int16(n) => {
                normalized = n;
                as_int = true;
                BGFX_ATTRIB_TYPE_INT16
            }
            AttribType::Half => BGFX_ATTRIB_TYPE_HALF,
            AttribType::Float => BGFX_ATTRIB_TYPE_FLOAT,
        };

        unsafe {
            bgfx_vertex_decl_add(&mut self.decl,
                                           mem::transmute(attrib),
                                           count,
                                           kind,
                                           normalized,
                                           as_int);
        }

        self
    }

    /// Finalizes the construction of the [`VertexDecl`].
    ///
    /// [`VertexDecl`]: struct.VertexDecl.html
    #[inline]
    pub fn end(&mut self) -> VertexDecl {
        unsafe {
            bgfx_vertex_decl_end(&mut self.decl);
        }

        VertexDecl { decl: self.decl }
    }

    /// Indicates a gap in the vertex structure.
    #[inline]
    pub fn skip(&mut self, bytes: u8) -> &mut Self {
        unsafe {
            bgfx_vertex_decl_skip(&mut self.decl, bytes);
        }

        self
    }

}

/// Acts as the library wrapper for bgfx. Any calls intended to be run on the main thread are
/// exposed as functions on this object.
///
/// It is created through a call to [`bgfx::init`], and will shut down bgfx when dropped.
///
/// [`bgfx::init`]: fn.init.html
pub struct Bgfx {
    // This dummy field only exists so this type can't be publicly instantiated.
    _dummy: u32,
}

impl Bgfx {

    #[inline]
    fn new() -> Bgfx {
        Bgfx { _dummy: 0 }
    }

    /// Clears the debug text overlay.
    #[inline]
    pub fn dbg_text_clear(&self, attr: Option<u8>, small: Option<bool>) {
        let attr = attr.unwrap_or(0);
        unsafe { bgfx_dbg_text_clear(attr, small.unwrap_or(false)) }
    }

    /// Draws an image to the debug text overlay.
    #[inline]
    pub fn dbg_text_image(&self,
                          x: u16,
                          y: u16,
                          width: u16,
                          height: u16,
                          data: &[u8],
                          pitch: u16) {
        unsafe {
            bgfx_dbg_text_image(x,
                                          y,
                                          width,
                                          height,
                                          data.as_ptr() as *const std::os::raw::c_void,
                                          pitch)
        }
    }

    /// Displays text in the debug text overlay.
    #[inline]
    pub fn dbg_text_print(&self, x: u16, y: u16, attr: u8, text: &str) {
        let text = ffi::CString::new(text).unwrap();
        unsafe { bgfx_dbg_text_printf(x, y, attr, text.as_ptr()) }
    }

    /// Finish the frame, syncing up with the render thread. Returns an incrementing frame counter.
    #[inline]
    pub fn frame(&self, capture: bool) -> u32 {
        unsafe { bgfx_frame(capture) }
    }

    /// Gets the type of the renderer in use.
    #[inline]
    pub fn get_renderer_type(&self) -> RendererType {
        unsafe { mem::transmute(bgfx_get_renderer_type()) }
    }

    /// Resets the graphics device to the given size, with the given flags.
    #[inline]
    pub fn reset(&self, width: u16, height: u16, flags: ResetFlags, format: TextureFormat) {
        unsafe { bgfx_reset(width as u32, height as u32, flags.bits(), format as i32) }
    }

    /// Sets the debug flags to use.
    #[inline]
    pub fn set_debug(&self, debug: DebugFlags) {
        unsafe { bgfx_set_debug(debug.bits()) }
    }

    /// Sets the index buffer to use for rendering.
    #[inline]
    pub fn set_index_buffer(&self, ibh: &IndexBuffer) {
        // TODO: How to solve lifetimes...
        unsafe { bgfx_set_index_buffer(ibh.handle, 0, std::u32::MAX) }
    }

    /// Sets the render state.
    #[inline]
    pub fn set_state(&self, state: StateFlags, rgba: Option<u32>) {
        unsafe { bgfx_set_state(state.bits(), rgba.unwrap_or(0)) }
    }

    /// Sets the model transform for rendering. If not called before submitting a draw, an identity
    /// matrix will be used.
    #[inline]
    pub fn set_transform(&self, mtx: &[f32; 16]) {
        unsafe {
            bgfx_set_transform(mtx.as_ptr() as *const std::os::raw::c_void, 1);
        }
    }

    /// Sets the vertex buffer to use for rendering.
    #[inline]
    pub fn set_vertex_buffer(&self, stream: u8, vbh: &VertexBuffer) {
        // TODO: How to solve lifetimes...
        unsafe { bgfx_set_vertex_buffer(stream, vbh.handle, 0, std::u32::MAX) }
    }

    /// Sets the options to use when clearing the given view.
    #[inline]
    pub fn set_view_clear(&self, id: ViewId, flags: ClearFlags, rgba: u32, depth: f32, stencil: u8) {
        unsafe { bgfx_set_view_clear(id, flags.bits(), rgba, depth, stencil) }
    }

    /// Sets the rectangle to display the given view in.
    #[inline]
    pub fn set_view_rect(&self, id: ViewId, x: u16, y: u16, width: u16, height: u16) {
        unsafe { bgfx_set_view_rect(id, x, y, width, height) }
    }

    /// Sets the view and projection matrices for the given view.
    #[inline]
    pub fn set_view_transform(&self, id: ViewId, view: &[f32; 16], proj: &[f32; 16]) {
        unsafe {
            bgfx_set_view_transform(id,
                                              view.as_ptr() as *const std::os::raw::c_void,
                                              proj.as_ptr() as *const std::os::raw::c_void)
        }
    }

    /// Submit a primitive for rendering. Returns the number of draw calls used.
    #[inline]
    pub fn submit(&self, view: ViewId, program: &Program, preserve_state: bool) {
        unsafe { bgfx_submit(view, program.handle, 0, preserve_state) }
    }

    /// Touches a view. ( ͡° ͜ʖ ͡°)
    #[inline]
    pub fn touch(&self, id: ViewId) {
        unsafe {
            bgfx_touch(id);
        }
    }

}

impl Drop for Bgfx {

    #[inline]
    fn drop(&mut self) {
        unsafe { bgfx_shutdown() }
    }

}

/// Pump the render thread.
///
/// This should be called repeatedly on the render thread.
#[inline]
pub fn render_frame(msecs: i32) -> RenderFrame {
    unsafe { mem::transmute(bgfx_render_frame(msecs)) }
}

/// Platform data initializer.
///
/// This should be applied *only once*, before bgfx is used.
///
/// # Example
///
/// ```should_panic
/// // Note: The default value for all of these options is null. If that is what you want, you may
/// // choose not to call said setter.
/// bgfx::PlatformData::new()
///     .context(std::ptr::null_mut())
///     .display(std::ptr::null_mut()) // Must be non-null on unix platforms
///     .window(std::ptr::null_mut()) // Must be non-null
///     .apply()
///     .expect("Could not set platform data");
/// ```
#[repr(transparent)]
pub struct PlatformData(bgfx_platform_data_t);

impl PlatformData {
    /// Creates an empty PlatformData instance.
    #[inline]
    pub fn new() -> Self {
        Self(bgfx_platform_data_t {
            ndt: ptr::null_mut(),
            nwh: ptr::null_mut(),
            context: ptr::null_mut(),
            backBuffer: ptr::null_mut(),
            backBufferDS: ptr::null_mut(),
        })
    }

    /// Apply the platform configuration.
    pub fn apply(&mut self) -> Result<(), BgfxError> {
        if self.0.ndt == ptr::null_mut() && cfg!(target_os = "linux") {
            Err(BgfxError::InvalidDisplay)
        } else if self.0.nwh == ptr::null_mut() {
            Err(BgfxError::InvalidWindow)
        } else {
            unsafe {
                bgfx_set_platform_data(&mut self.0);
            }
            Ok(())
        }
    }

    /// Sets the GL context to use.
    #[inline]
    pub fn context(&mut self, context: *mut std::os::raw::c_void) -> &mut Self {
        self.0.context = context;
        self
    }

    /// Sets the X11 display to use on unix systems.
    #[inline]
    pub fn display(&mut self, display: *mut std::os::raw::c_void) -> &mut Self {
        self.0.ndt = display;
        self
    }

    /// Sets the handle to the window to use.
    #[inline]
    pub fn window(&mut self, window: *mut std::os::raw::c_void) -> &mut Self {
        self.0.nwh = window;
        self
    }

}

#[repr(transparent)]
pub struct Init(bgfx_init_t);

impl Init {
    pub fn init(self) -> Result<Bgfx, BgfxError> { crate::init(&self) }
    pub fn with_renderer(mut self, renderer: RendererType) -> Self { self.0.type_ = renderer as i32; self }
    pub fn with_resolution(mut self, resolution: Resolution) -> Self { self.0.resolution = resolution.0; self }
    pub fn with_limits(mut self, limits: InitLimits) -> Self { self.0.limits = limits.0; self }
}

impl Default for Init {
    fn default() -> Self {
        let mut init = bgfx_init_t{
            type_:          RendererType::Default as i32,
            vendorId:       PCI_ID_NONE,
            deviceId:       0,
            debug:          cfg!(debug),
            profile:        cfg!(debug),
            platformData:   PlatformData::new().0,
            resolution:     Resolution::default().0,
            limits:         InitLimits::default().0,
            callback:       ptr::null_mut(),
            allocator:      ptr::null_mut(),
        };
        unsafe { bgfx_init_ctor(&mut init); }
        Self(init)
    }
}

/// Initializes bgfx.
///
/// This must be called on the main thread after setting the platform data. See [`PlatformData`].
///
/// [`PlatformData`]: struct.PlatformData.html
pub fn init(init: &Init) -> Result<Bgfx, BgfxError> {
    unsafe {
        let success = bgfx_init(&init.0);
        if success { Ok(Bgfx::new()) } else { Err(BgfxError::InitFailed) }
    }
}
