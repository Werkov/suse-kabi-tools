#
# spec file for package suse-kabi-tools
#
# Copyright (c) 2024 SUSE LLC
#
# All modifications and additions to the file contributed by third parties
# remain the property of their copyright owners, unless otherwise agreed
# upon. The license for this file, and modifications and additions to the
# file, is the same license as for the pristine package itself (unless the
# license for the pristine package is not an Open Source License, in which
# case the license is the MIT License). An "Open Source License" is a
# license that conforms to the Open Source Definition (Version 1.9)
# published by the Open Source Initiative.

# Please submit bugfixes or comments via https://bugs.opensuse.org/
#


Name:           suse-kabi-tools
Version:        2025
Release:        0
Summary:        Tools for extracting and comparing SUSE kernels' ABI
License:        GPL-2.0-only
Group:          System/Kernel
Source:		%{name}-%{version}.tar.gz
Requires:       perl

%description


%prep
%setup -q

%build

%install
echo $PWD
pwd
install -m0755 -D kabi.pl %{buildroot}/%{_bindir}/suse-kabi

%files
%{_bindir}/suse-kabi

%changelog
