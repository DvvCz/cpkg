repo=DvvCz/cpkg
triple=x86_64-unknown-linux-gnu

if command -v cpkg &> /dev/null; then
	echo "Failed to install cpkg -- already exists."
	exit 1
fi

# Get latest release
release_json=$(curl -s https://api.github.com/repos/$repo/releases/latest)
release_url=$(grep -o "https://.*/cpkg-v.*-$triple" <<< $release_json)
release_filename=$(grep -o "cpkg-v.*" <<< $release_url)

# CPKG_DIR defaults to ~/.cpkg
cpkg_dir=${CPKG_DIR:=~/.cpkg}

if ! test -v $cpkg_dir; then
	mkdir $cpkg_dir
fi

if ! test -v $cpkg_dir/bin; then
	mkdir $cpkg_dir/bin
fi

# wget handles folder creation for you
wget -qi - <<< $release_url -O $cpkg_dir/bin/cpkg

chmod +x $cpkg_dir/bin/cpkg

echo -e "\nexport CPKG_INSTALL=\"$cpkg_dir\"" >> ~/.bashrc
echo -e 'export PATH=$CPKG_INSTALL/bin:$PATH\n' >> ~/.bashrc

echo "Installed cpkg to $cpkg_dir and to PATH in ~/.bashrc.\nYou may need to run source ~/.bashrc."