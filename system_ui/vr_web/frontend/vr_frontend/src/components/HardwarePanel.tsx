import React, { useState, useEffect } from 'react';
import { hardwareApi } from '@/lib/api';
import { 
  Table, 
  TableBody, 
  TableCaption, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from "@/components/ui/table";
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ReloadIcon } from "@radix-ui/react-icons";
import { 
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";

interface HardwareDevice {
  name: string;
  device_type: string;
  status: string;
  properties: Record<string, string>;
}

interface DiagnosticResult {
  test_name: string;
  status: string;
  message: string;
}

export const HardwarePanel: React.FC = () => {
  const [devices, setDevices] = useState<HardwareDevice[]>([]);
  const [selectedType, setSelectedType] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedDevice, setSelectedDevice] = useState<HardwareDevice | null>(null);
  const [diagnosticResults, setDiagnosticResults] = useState<DiagnosticResult[]>([]);
  const [diagnosisLoading, setDiagnosisLoading] = useState<boolean>(false);

  const deviceTypes = [
    { value: '', label: 'All Devices' },
    { value: 'camera', label: 'Cameras' },
    { value: 'imu', label: 'IMUs' },
    { value: 'display', label: 'Displays' },
  ];

  useEffect(() => {
    fetchDevices();
  }, [selectedType]);

  const fetchDevices = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await hardwareApi.listHardware(selectedType || undefined);
      setDevices(response.devices);
    } catch (err) {
      console.error('Failed to fetch hardware devices:', err);
      setError('Failed to load hardware devices. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleTypeChange = (value: string) => {
    setSelectedType(value);
  };

  const handleInitializeAll = async () => {
    setLoading(true);
    setError(null);
    try {
      await hardwareApi.initHardware();
      // Refresh the list
      fetchDevices();
    } catch (err) {
      console.error('Failed to initialize hardware:', err);
      setError('Failed to initialize hardware. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleShutdownAll = async () => {
    if (!window.confirm('Are you sure you want to shutdown all hardware devices?')) {
      return;
    }
    
    setLoading(true);
    setError(null);
    try {
      await hardwareApi.shutdownHardware();
      // Refresh the list
      fetchDevices();
    } catch (err) {
      console.error('Failed to shutdown hardware:', err);
      setError('Failed to shutdown hardware. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleInitializeDevice = async (device: HardwareDevice) => {
    setLoading(true);
    setError(null);
    try {
      await hardwareApi.initHardware(device.name);
      // Refresh the list
      fetchDevices();
    } catch (err) {
      console.error(`Failed to initialize device ${device.name}:`, err);
      setError(`Failed to initialize device ${device.name}. Please try again.`);
    } finally {
      setLoading(false);
    }
  };

  const handleShutdownDevice = async (device: HardwareDevice) => {
    if (!window.confirm(`Are you sure you want to shutdown ${device.name}?`)) {
      return;
    }
    
    setLoading(true);
    setError(null);
    try {
      await hardwareApi.shutdownHardware(device.name);
      // Refresh the list
      fetchDevices();
    } catch (err) {
      console.error(`Failed to shutdown device ${device.name}:`, err);
      setError(`Failed to shutdown device ${device.name}. Please try again.`);
    } finally {
      setLoading(false);
    }
  };

  const handleDiagnoseDevice = async (device: HardwareDevice) => {
    setSelectedDevice(device);
    setDiagnosisLoading(true);
    setDiagnosticResults([]);
    try {
      const response = await hardwareApi.diagnoseHardware(device.name, 'full');
      setDiagnosticResults(response.results);
    } catch (err) {
      console.error(`Failed to diagnose device ${device.name}:`, err);
      setError(`Failed to diagnose device ${device.name}. Please try again.`);
    } finally {
      setDiagnosisLoading(false);
    }
  };

  const getStatusBadge = (status: string) => {
    if (status === 'initialized') {
      return <Badge variant="success">Initialized</Badge>;
    } else if (status === 'not initialized') {
      return <Badge variant="destructive">Not Initialized</Badge>;
    } else {
      return <Badge variant="outline">{status}</Badge>;
    }
  };

  const getDiagnosticStatusBadge = (status: string) => {
    if (status === 'passed') {
      return <Badge variant="success">Passed</Badge>;
    } else if (status === 'warning') {
      return <Badge variant="warning">Warning</Badge>;
    } else if (status === 'failed') {
      return <Badge variant="destructive">Failed</Badge>;
    } else {
      return <Badge variant="outline">{status}</Badge>;
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-4">
          <Select value={selectedType} onValueChange={handleTypeChange}>
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder="Select Device Type" />
            </SelectTrigger>
            <SelectContent>
              {deviceTypes.map((type) => (
                <SelectItem key={type.value} value={type.value}>
                  {type.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          
          <Button 
            variant="outline" 
            size="sm" 
            onClick={fetchDevices}
            disabled={loading}
          >
            {loading ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Loading...
              </>
            ) : (
              'Refresh'
            )}
          </Button>
        </div>
        
        <div className="flex space-x-2">
          <Button 
            variant="default" 
            size="sm" 
            onClick={handleInitializeAll}
            disabled={loading}
          >
            Initialize All
          </Button>
          <Button 
            variant="destructive" 
            size="sm" 
            onClick={handleShutdownAll}
            disabled={loading}
          >
            Shutdown All
          </Button>
        </div>
      </div>
      
      {error && (
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      
      <Table>
        <TableCaption>VR Headset Hardware Devices</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Properties</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {loading && devices.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center">
                <ReloadIcon className="h-4 w-4 animate-spin inline mr-2" />
                Loading devices...
              </TableCell>
            </TableRow>
          ) : devices.length === 0 ? (
            <TableRow>
              <TableCell colSpan={5} className="text-center">
                No hardware devices found.
              </TableCell>
            </TableRow>
          ) : (
            devices.map((device, index) => (
              <TableRow key={`${device.name}-${index}`}>
                <TableCell>{device.name}</TableCell>
                <TableCell>{device.device_type}</TableCell>
                <TableCell>{getStatusBadge(device.status)}</TableCell>
                <TableCell>
                  {Object.entries(device.properties).map(([key, value]) => (
                    <div key={key}>
                      <span className="font-medium">{key}:</span> {value}
                    </div>
                  ))}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end space-x-2">
                    {device.status === 'initialized' ? (
                      <Button 
                        variant="destructive" 
                        size="sm" 
                        onClick={() => handleShutdownDevice(device)}
                        disabled={loading}
                      >
                        Shutdown
                      </Button>
                    ) : (
                      <Button 
                        variant="default" 
                        size="sm" 
                        onClick={() => handleInitializeDevice(device)}
                        disabled={loading}
                      >
                        Initialize
                      </Button>
                    )}
                    
                    <Dialog>
                      <DialogTrigger asChild>
                        <Button 
                          variant="outline" 
                          size="sm" 
                          onClick={() => handleDiagnoseDevice(device)}
                          disabled={loading}
                        >
                          Diagnose
                        </Button>
                      </DialogTrigger>
                      <DialogContent className="sm:max-w-[425px]">
                        <DialogHeader>
                          <DialogTitle>Diagnostic Results: {selectedDevice?.name}</DialogTitle>
                          <DialogDescription>
                            Detailed diagnostic information for this device.
                          </DialogDescription>
                        </DialogHeader>
                        
                        {diagnosisLoading ? (
                          <div className="flex justify-center py-4">
                            <ReloadIcon className="h-6 w-6 animate-spin" />
                          </div>
                        ) : diagnosticResults.length === 0 ? (
                          <div className="text-center py-4">
                            No diagnostic results available.
                          </div>
                        ) : (
                          <div className="space-y-4 mt-4">
                            {diagnosticResults.map((result, index) => (
                              <div key={index} className="border rounded-md p-3">
                                <div className="flex justify-between items-center mb-2">
                                  <h4 className="font-medium">{result.test_name}</h4>
                                  {getDiagnosticStatusBadge(result.status)}
                                </div>
                                <p className="text-sm text-muted-foreground">{result.message}</p>
                              </div>
                            ))}
                          </div>
                        )}
                      </DialogContent>
                    </Dialog>
                  </div>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
};
