//! Meta data for dealing with input / output channels. Not all hosts use this so it is not
//! necessary for plugin functionality.

use crate::api;
use crate::api::consts::{MAX_LABEL, MAX_SHORT_LABEL};

/// Information about an input / output channel. This isn't necessary for a channel to function but
/// informs the host how the channel is meant to be used.
pub struct ChannelInfo {
    name: String,
    short_name: String,
    active: bool,
    arrangement_type: SpeakerArrangementType,
}

impl ChannelInfo {
    /// Construct a new `ChannelInfo` object.
    ///
    /// `name` is a user friendly name for this channel limited to `MAX_LABEL` characters.
    /// `short_name` is an optional field which provides a short name limited to `MAX_SHORT_LABEL`.
    /// `active` determines whether this channel is active.
    /// `arrangement_type` describes the arrangement type for this channel.
    pub fn new(
        name: String,
        short_name: Option<String>,
        active: bool,
        arrangement_type: Option<SpeakerArrangementType>,
    ) -> ChannelInfo {
        ChannelInfo {
            name: name.clone(),

            short_name: if let Some(short_name) = short_name {
                short_name
            } else {
                name
            },

            active,

            arrangement_type: arrangement_type.unwrap_or(SpeakerArrangementType::Custom),
        }
    }
}

impl Into<api::ChannelProperties> for ChannelInfo {
    /// Convert to the VST api equivalent of this structure.
    fn into(self) -> api::ChannelProperties {
        api::ChannelProperties {
            name: {
                let mut label = [0; MAX_LABEL as usize];
                for (b, c) in self.name.bytes().zip(label.iter_mut()) {
                    *c = b;
                }
                label
            },
            flags: {
                let mut flag = api::ChannelFlags::empty();
                if self.active {
                    flag |= api::ChannelFlags::ACTIVE
                }
                if self.arrangement_type.is_left_stereo() {
                    flag |= api::ChannelFlags::STEREO
                }
                if self.arrangement_type.is_speaker_type() {
                    flag |= api::ChannelFlags::SPEAKER
                }
                flag.bits()
            },
            arrangement_type: self.arrangement_type.into(),
            short_name: {
                let mut label = [0; MAX_SHORT_LABEL as usize];
                for (b, c) in self.short_name.bytes().zip(label.iter_mut()) {
                    *c = b;
                }
                label
            },
            future: [0; 48],
        }
    }
}

impl From<api::ChannelProperties> for ChannelInfo {
    fn from(api: api::ChannelProperties) -> ChannelInfo {
        ChannelInfo {
            name: String::from_utf8_lossy(&api.name).to_string(),
            short_name: String::from_utf8_lossy(&api.short_name).to_string(),
            active: api::ChannelFlags::from_bits(api.flags)
                .expect("Invalid bits in channel info")
                .intersects(api::ChannelFlags::ACTIVE),
            arrangement_type: SpeakerArrangementType::from(api),
        }
    }
}

/// Target for Speaker arrangement type. Can be a cinema configuration or music configuration. Both
/// are technically identical but this provides extra information to the host.
pub enum ArrangementTarget {
    /// Music arrangement. Technically identical to Cinema.
    Music,
    /// Cinematic arrangement. Technically identical to Music.
    Cinema,
}

/// An enum for all channels in a stereo configuration.
pub enum StereoChannel {
    /// Left channel.
    Left,
    /// Right channel.
    Right,
}

/// Possible stereo speaker configurations.
#[allow(non_camel_case_types)]
pub enum StereoConfig {
    /// Regular.
    L_R,
    /// Left surround, right surround.
    Ls_Rs,
    /// Left center, right center.
    Lc_Rc,
    /// Side left, side right.
    Sl_Sr,
    /// Center, low frequency effects.
    C_Lfe,
}

/// Possible surround speaker configurations.
#[allow(non_camel_case_types)]
pub enum SurroundConfig {
    /// 3.0 surround sound.
    /// Cinema: L R C
    /// Music: L R S
    S3_0(ArrangementTarget),
    /// 3.1 surround sound.
    /// Cinema: L R C Lfe
    /// Music: L R Lfe S
    S3_1(ArrangementTarget),
    /// 4.0 surround sound.
    /// Cinema: L R C S (LCRS)
    /// Music: L R Ls Rs (Quadro)
    S4_0(ArrangementTarget),
    /// 4.1 surround sound.
    /// Cinema: L R C Lfe S (LCRS + Lfe)
    /// Music: L R Ls Rs (Quadro + Lfe)
    S4_1(ArrangementTarget),
    /// 5.0 surround sound.
    /// Cinema and music: L R C Ls Rs
    S5_0,
    /// 5.1 surround sound.
    /// Cinema and music: L R C Lfe Ls Rs
    S5_1,
    /// 6.0 surround sound.
    /// Cinema: L R C Ls Rs Cs
    /// Music: L R Ls Rs Sl Sr
    S6_0(ArrangementTarget),
    /// 6.1 surround sound.
    /// Cinema: L R C Lfe Ls Rs Cs
    /// Music: L R Ls Rs Sl Sr
    S6_1(ArrangementTarget),
    /// 7.0 surround sound.
    /// Cinema: L R C Ls Rs Lc Rc
    /// Music: L R C Ls Rs Sl Sr
    S7_0(ArrangementTarget),
    /// 7.1 surround sound.
    /// Cinema: L R C Lfe Ls Rs Lc Rc
    /// Music: L R C Lfe Ls Rs Sl Sr
    S7_1(ArrangementTarget),
    /// 8.0 surround sound.
    /// Cinema: L R C Ls Rs Lc Rc Cs
    /// Music: L R C Ls Rs Cs Sl Sr
    S8_0(ArrangementTarget),
    /// 8.1 surround sound.
    /// Cinema: L R C Lfe Ls Rs Lc Rc Cs
    /// Music: L R C Lfe Ls Rs Cs Sl Sr
    S8_1(ArrangementTarget),
    /// 10.2 surround sound.
    /// Cinema + Music: L R C Lfe Ls Rs Tfl Tfc Tfr Trl Trr Lfe2
    S10_2,
}

/// Type representing how a channel is used. Only useful for some hosts.
pub enum SpeakerArrangementType {
    /// Custom arrangement not specified to host.
    Custom,
    /// Empty arrangement.
    Empty,
    /// Mono channel.
    Mono,
    /// Stereo channel. Contains type of stereo arrangement and speaker represented.
    Stereo(StereoConfig, StereoChannel),
    /// Surround channel. Contains surround arrangement and target (cinema or music).
    Surround(SurroundConfig),
}

impl Default for SpeakerArrangementType {
    fn default() -> SpeakerArrangementType {
        SpeakerArrangementType::Mono
    }
}

impl SpeakerArrangementType {
    /// Determine whether this channel is part of a surround speaker arrangement.
    pub fn is_speaker_type(&self) -> bool {
        if let SpeakerArrangementType::Surround(..) = *self {
            true
        } else {
            false
        }
    }

    /// Determine whether this channel is the left speaker in a stereo pair.
    pub fn is_left_stereo(&self) -> bool {
        if let SpeakerArrangementType::Stereo(_, StereoChannel::Left) = *self {
            true
        } else {
            false
        }
    }
}

impl Into<api::SpeakerArrangementType> for SpeakerArrangementType {
    /// Convert to VST API arrangement type.
    fn into(self) -> api::SpeakerArrangementType {
        use self::ArrangementTarget::{Cinema, Music};
        use self::SpeakerArrangementType::*;
        use api::SpeakerArrangementType as Raw;

        match self {
            Custom => Raw::Custom,
            Empty => Raw::Empty,
            Mono => Raw::Mono,
            Stereo(conf, _) => {
                match conf {
                    // Stereo channels.
                    StereoConfig::L_R => Raw::Stereo,
                    StereoConfig::Ls_Rs => Raw::StereoSurround,
                    StereoConfig::Lc_Rc => Raw::StereoCenter,
                    StereoConfig::Sl_Sr => Raw::StereoSide,
                    StereoConfig::C_Lfe => Raw::StereoCLfe,
                }
            }
            Surround(conf) => {
                match conf {
                    // Surround channels.
                    SurroundConfig::S3_0(Music) => Raw::Music30,
                    SurroundConfig::S3_0(Cinema) => Raw::Cinema30,

                    SurroundConfig::S3_1(Music) => Raw::Music31,
                    SurroundConfig::S3_1(Cinema) => Raw::Cinema31,

                    SurroundConfig::S4_0(Music) => Raw::Music40,
                    SurroundConfig::S4_0(Cinema) => Raw::Cinema40,

                    SurroundConfig::S4_1(Music) => Raw::Music41,
                    SurroundConfig::S4_1(Cinema) => Raw::Cinema41,

                    SurroundConfig::S5_0 => Raw::Surround50,
                    SurroundConfig::S5_1 => Raw::Surround51,

                    SurroundConfig::S6_0(Music) => Raw::Music60,
                    SurroundConfig::S6_0(Cinema) => Raw::Cinema60,

                    SurroundConfig::S6_1(Music) => Raw::Music61,
                    SurroundConfig::S6_1(Cinema) => Raw::Cinema61,

                    SurroundConfig::S7_0(Music) => Raw::Music70,
                    SurroundConfig::S7_0(Cinema) => Raw::Cinema70,

                    SurroundConfig::S7_1(Music) => Raw::Music71,
                    SurroundConfig::S7_1(Cinema) => Raw::Cinema71,

                    SurroundConfig::S8_0(Music) => Raw::Music80,
                    SurroundConfig::S8_0(Cinema) => Raw::Cinema80,

                    SurroundConfig::S8_1(Music) => Raw::Music81,
                    SurroundConfig::S8_1(Cinema) => Raw::Cinema81,

                    SurroundConfig::S10_2 => Raw::Surround102,
                }
            }
        }
    }
}

/// Convert the VST API equivalent struct into something more usable.
///
/// We implement `From<ChannelProperties>` as `SpeakerArrangementType` contains extra info about
/// stereo speakers found in the channel flags.
impl From<api::ChannelProperties> for SpeakerArrangementType {
    fn from(api: api::ChannelProperties) -> SpeakerArrangementType {
        use self::ArrangementTarget::{Cinema, Music};
        use self::SpeakerArrangementType::*;
        use self::SurroundConfig::*;
        use api::SpeakerArrangementType as Raw;

        let stereo = if api::ChannelFlags::from_bits(api.flags)
            .expect("Invalid Channel Flags")
            .intersects(api::ChannelFlags::STEREO)
        {
            StereoChannel::Left
        } else {
            StereoChannel::Right
        };

        match api.arrangement_type {
            Raw::Custom => Custom,
            Raw::Empty => Empty,
            Raw::Mono => Mono,

            Raw::Stereo => Stereo(StereoConfig::L_R, stereo),
            Raw::StereoSurround => Stereo(StereoConfig::Ls_Rs, stereo),
            Raw::StereoCenter => Stereo(StereoConfig::Lc_Rc, stereo),
            Raw::StereoSide => Stereo(StereoConfig::Sl_Sr, stereo),
            Raw::StereoCLfe => Stereo(StereoConfig::C_Lfe, stereo),

            Raw::Music30 => Surround(S3_0(Music)),
            Raw::Cinema30 => Surround(S3_0(Cinema)),

            Raw::Music31 => Surround(S3_1(Music)),
            Raw::Cinema31 => Surround(S3_1(Cinema)),

            Raw::Music40 => Surround(S4_0(Music)),
            Raw::Cinema40 => Surround(S4_0(Cinema)),

            Raw::Music41 => Surround(S4_1(Music)),
            Raw::Cinema41 => Surround(S4_1(Cinema)),

            Raw::Surround50 => Surround(S5_0),
            Raw::Surround51 => Surround(S5_1),

            Raw::Music60 => Surround(S6_0(Music)),
            Raw::Cinema60 => Surround(S6_0(Cinema)),

            Raw::Music61 => Surround(S6_1(Music)),
            Raw::Cinema61 => Surround(S6_1(Cinema)),

            Raw::Music70 => Surround(S7_0(Music)),
            Raw::Cinema70 => Surround(S7_0(Cinema)),

            Raw::Music71 => Surround(S7_1(Music)),
            Raw::Cinema71 => Surround(S7_1(Cinema)),

            Raw::Music80 => Surround(S8_0(Music)),
            Raw::Cinema80 => Surround(S8_0(Cinema)),

            Raw::Music81 => Surround(S8_1(Music)),
            Raw::Cinema81 => Surround(S8_1(Cinema)),

            Raw::Surround102 => Surround(S10_2),
        }
    }
}
