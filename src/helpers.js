let resized = false;

export async function js_track_resized_setup() {
    window.addEventListener("resize", () => {
        resized = true;
    });
};

export function js_poll_resized() {
    let ret = resized;
    resized = false;
    return ret;
}
