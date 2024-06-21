
fn setup() {
	let rustup = cmd(["rustup", "target", "install", "thumbv7m-none-eabi"]);
	rustup.execute();

	// install binutils-arm-linux-gnueabi
	let apt = cmd([
		"sudo", "apt", "install", "binutils-arm-linux-gnueabi"
	]);
}

fn _crate_name() {
	return toml::read("./Cargo.toml")["package"].name;
}

fn _target_dir() {
	return "./target/thumbv7m-none-eabi/release";
}

fn build() {
	let cargo = cmd(["cargo", "build", "--release"]);
	cargo.execute();

	let name = _crate_name();
	let target_dir = _target_dir();

	// copy obj to bin
	let objcopy = cmd([
		"arm-linux-gnueabi-objcopy",
		"-O", "binary",
		target_dir + "/" + name,
		target_dir + "/" + name + ".bin"
	]);
	objcopy.execute();
}

fn flash() {
	let name = _crate_name();
	let target_dir = _target_dir();

	let dfu = cmd([
		"dfu-util", "-a", "0", "-s", "0x08000000:leave",
		"-D", target_dir + "/" + name + ".bin"
	]);
	dfu.execute();
}

fn build_flash() {
	build();
	flash();
}