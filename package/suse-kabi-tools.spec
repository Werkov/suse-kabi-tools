#
# spec file for package suse-kabi-tools
#
# Copyright (c) 2025 SUSE LLC
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
Version:        0.2.0+git5.9801863
Release:        0
Summary:        A set of ABI tools for the Linux kernel
License:        GPL-2.0-or-later
URL:            https://github.com/SUSE/suse-kabi-tools
Source:         %{name}-%{version}.tar.zst
BuildRequires:  cargo
BuildRequires:  cargo-packaging

%description
suse-kabi-tools is a set of Application Binary Interface (ABI) tools for the
Linux kernel.

%prep
%autosetup -p1

%build
%{cargo_build}

%install
%{cargo_install}
install -D -m 0644 %{_builddir}/%{name}-%{version}/doc/ksymtypes.1 %{buildroot}%{_mandir}/man1/ksymtypes.1
install -D -m 0644 %{_builddir}/%{name}-%{version}/doc/ksymtypes.5 %{buildroot}%{_mandir}/man5/ksymtypes.5

%check
%{cargo_test}

%files
%license COPYING
%{_bindir}/ksymtypes
%{_mandir}/man1/ksymtypes.1%{?ext_man}
%{_mandir}/man5/ksymtypes.5%{?ext_man}

%changelog

