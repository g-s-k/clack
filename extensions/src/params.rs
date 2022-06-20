use clack_common::extensions::{Extension, PluginExtension};
use clap_sys::ext::params::{clap_plugin_params, CLAP_EXT_PARAMS};
use std::marker::PhantomData;
use std::os::raw::c_char;

#[repr(C)]
pub struct PluginParams(clap_plugin_params, PhantomData<*const clap_plugin_params>);

#[cfg(feature = "clack-plugin")]
pub mod implementation;
pub mod info;

unsafe impl Extension for PluginParams {
    const IDENTIFIER: *const c_char = CLAP_EXT_PARAMS;
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use crate::params::info::ParamInfo;
    use clack_common::events::io::{InputEvents, OutputEvents};
    use clack_host::host::PluginHoster;
    use clack_host::instance::processor::StoppedPluginAudioProcessor;
    use clack_host::instance::PluginInstance;
    use std::ffi::CStr;
    use std::mem::MaybeUninit;

    impl PluginParams {
        pub fn count<'a, H: PluginHoster<'a>>(&self, plugin: &PluginInstance<'a, H>) -> u32 {
            if let Some(count) = self.0.count {
                unsafe { count(plugin.raw_instance()) }
            } else {
                0
            }
        }

        pub fn get_info<'a, 'b, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            param_index: u32,
            info: &'b mut MaybeUninit<ParamInfo>,
        ) -> Option<&'b mut ParamInfo> {
            let valid = if let Some(get_info) = self.0.get_info {
                unsafe {
                    get_info(
                        plugin.raw_instance(),
                        param_index,
                        info.as_mut_ptr() as *mut _,
                    )
                }
            } else {
                return None;
            };

            if valid {
                unsafe { Some(info.assume_init_mut()) }
            } else {
                None
            }
        }

        pub fn get_value<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            param_id: u32,
        ) -> Option<f64> {
            let mut value = MaybeUninit::uninit();
            let valid = if let Some(get_value) = self.0.get_value {
                unsafe { get_value(plugin.raw_instance(), param_id, value.as_mut_ptr()) }
            } else {
                return None;
            };

            if valid {
                unsafe { Some(value.assume_init()) }
            } else {
                None
            }
        }

        pub fn value_to_text<'a, 'b, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            param_id: u32,
            value: f64,
            buffer: &'b mut [std::mem::MaybeUninit<u8>],
        ) -> Option<&'b mut [u8]> {
            let valid = if let Some(value_to_text) = self.0.value_to_text {
                unsafe {
                    value_to_text(
                        plugin.raw_instance(),
                        param_id,
                        value,
                        buffer.as_mut_ptr() as *mut _,
                        buffer.len() as u32,
                    )
                }
            } else {
                false
            };

            if valid {
                // SAFETY: technically not all of the buffer may be initialized, but uninit u8 is fine
                let buffer = unsafe { assume_init_slice(buffer) };
                // If no nul byte found, we take the entire buffer
                let buffer_total_len = buffer.iter().position(|b| *b == 0).unwrap_or(buffer.len());
                Some(&mut buffer[..buffer_total_len])
            } else {
                None
            }
        }

        pub fn text_to_value<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            param_id: u32,
            display: &CStr,
        ) -> Option<f64> {
            let mut value = MaybeUninit::uninit();

            let valid = if let Some(text_to_value) = self.0.text_to_value {
                unsafe {
                    text_to_value(
                        plugin.raw_instance(),
                        param_id,
                        display.as_ptr(),
                        value.as_mut_ptr(),
                    )
                }
            } else {
                false
            };

            if valid {
                unsafe { Some(value.assume_init()) }
            } else {
                None
            }
        }

        // TODO: return a proper error
        pub fn flush_inactive<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &mut PluginInstance<'a, H>,
            input_event_list: &InputEvents,
            output_event_list: &mut OutputEvents,
        ) -> bool {
            if plugin.is_active() {
                return false;
            }

            if let Some(flush) = self.0.flush {
                unsafe {
                    flush(
                        plugin.raw_instance(),
                        input_event_list.as_raw(),
                        output_event_list.as_raw_mut(),
                    )
                };
                true
            } else {
                false
            }
        }

        pub fn flush_active<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &mut StoppedPluginAudioProcessor<'a, H>, // TODO: separate handle type
            input_event_list: &InputEvents,
            output_event_list: &mut OutputEvents,
        ) {
            if let Some(flush) = self.0.flush {
                // SAFETY: flush is already guaranteed by the types to be called on an active, non-processing plugin
                unsafe {
                    flush(
                        plugin.audio_processor_plugin_data().as_raw(),
                        input_event_list.as_raw(),
                        output_event_list.as_raw_mut(),
                    )
                }
            }
        }
    }

    #[inline]
    unsafe fn assume_init_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
        &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
    }
}
