use super::*;

#[test]
fn research_tools_use_the_book_animation() {
    for name in ["read_file", "grep", "glob", "web_search", "web_fetch"] {
        assert_eq!(tool_animation(name), MascotAnimation::ExploreBook);
    }
}

#[test]
fn action_tools_use_the_laptop_animation() {
    for name in ["bash", "write_file", "edit_file", "delegate"] {
        assert_eq!(tool_animation(name), MascotAnimation::WorkLaptop);
    }
}
