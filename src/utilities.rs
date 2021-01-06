/// Returns a properly capitalized event name, or [None] if the original string was empty or didn't contain an event name
pub fn proper_event_name(original: &str) -> Option<&str> {
	match original.to_ascii_lowercase().as_ref() {
		"memberjoin" => {
			return Some("MemberJoin");
		}
		"memberleave" => {
			return Some("MemberLeave");
		}
		"memberupdate" => {
			return Some("MemberUpdate");
		}
		"rolecreate" => {
			return Some("RoleCreate");
		}
		"roleupdate" => {
			return Some("RoleUpdate");
		}
		"channelcreate" => {
			return Some("ChannelCreate");
		}
		"channeldelete" => {
			return Some("ChannelDelete");
		}
		"channelupdate" => {
			return Some("ChannelUpdate");
		}
		"guildupdate" => {
			return Some("GuildUpdate");
		}
		"voiceupdate" => {
			return Some("VoiceUpdate");
		}
		"reactionadd" => {
			return Some("ReactionAdd");
		}
		"reactionremove" => {
			return Some("ReactionRemove");
		}
		_ => {
			return None;
		}
	};
}