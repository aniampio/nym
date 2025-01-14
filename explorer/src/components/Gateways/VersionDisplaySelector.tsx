import React from 'react';
import { FormControl, MenuItem, Select } from '@mui/material';
import { useIsMobile } from '../../hooks/useIsMobile';

export enum VersionSelectOptions {
  latestVersion = 'Latest versions',
  olderVersions = 'Older versions',
}
export const VersionDisplaySelector = ({
  selected,
  handleChange,
}: {
  selected: VersionSelectOptions;
  handleChange: (option: VersionSelectOptions) => void;
}) => {
  const isMobile = useIsMobile();

  return (
    <FormControl size="small">
      <Select
        value={selected}
        onChange={(e) => handleChange(e.target.value as VersionSelectOptions)}
        labelId="simple-select-label"
        id="simple-select"
        sx={{
          marginRight: isMobile ? 0 : 2,
        }}
      >
        <MenuItem value={VersionSelectOptions.latestVersion} data-testid="show-gateway-latest-version">
          {VersionSelectOptions.latestVersion}
        </MenuItem>
        <MenuItem value={VersionSelectOptions.olderVersions} data-testid="show-gateway-old-versions">
          {VersionSelectOptions.olderVersions}
        </MenuItem>
      </Select>
    </FormControl>
  );
};
