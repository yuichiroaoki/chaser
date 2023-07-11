import React, { useEffect } from 'react';
import {
  Container,
  Flex,
  Box,
  Text,
  Button,
  VStack,
  HStack,
  Wrap,
  WrapItem,
  FormControl,
  FormLabel,
  Input,
  InputGroup,
  InputLeftElement,
} from '@chakra-ui/react';
import axios from 'axios';

function App() {
  const [tokenIn, setTokenIn] = React.useState('');
  const [tokenOut, setTokenOut] = React.useState('');
  useEffect(() => {
    const fetchData = async () => {
      axios.get(`http://localhost:8000/path/${tokenIn}/${tokenOut}`, {})
        .then((res) => {
          if (res.status === 200) {
            console.log(res.data);
          }
        })
        .catch((err) => {
          console.log(err);
        })
    };
    fetchData();

  }, [tokenIn, tokenOut]);

  return (
    <Container maxW="full" mt={0} centerContent overflow="hidden">
      <Flex>
        <Box
          bg="#02054B"
          color="white"
          borderRadius="lg"
          m={{ sm: 4, md: 16, lg: 10 }}
          p={{ sm: 5, md: 5, lg: 16 }}>
          <Box p={4}>
            <Wrap spacing={{ base: 20, sm: 3, md: 5, lg: 20 }}>
              <WrapItem>
                <Box>
                  <Box py={{ base: 5, sm: 5, md: 8, lg: 10 }}>
                    <VStack pl={0} spacing={3} alignItems="flex-start">
                      <Text
                        size="md"
                        height="48px"
                        width="200px"
                        color="#DCE2FF"
                      >
                      </Text>
                    </VStack>
                  </Box>
                  <HStack
                    mt={{ lg: 10, md: 10 }}
                    spacing={5}
                    px={5}
                    alignItems="flex-start">
                  </HStack>
                </Box>
              </WrapItem>
              <WrapItem>
                <Box bg="white" borderRadius="lg">
                  <Box m={8} color="#0B0E3F">
                    <VStack spacing={5}>
                      <FormControl id="name">
                        <FormLabel>token in</FormLabel>
                        <InputGroup borderColor="#E0E1E7">
                          <InputLeftElement
                            pointerEvents="none"
                          />
                          <Input type="text" size="md"
                            onChange={(e) => setTokenIn(e.target.value)}
                          />
                        </InputGroup>
                      </FormControl>
                      <FormControl id="name">
                        <FormLabel>token out</FormLabel>
                        <InputGroup borderColor="#E0E1E7">
                          <InputLeftElement
                            pointerEvents="none"
                          />
                          <Input type="text" size="md"
                            onChange={(e) => setTokenOut(e.target.value)}
                          />
                        </InputGroup>
                      </FormControl>
                      <FormControl id="name" float="right">
                        <Button
                          variant="solid"
                          bg="#0D74FF"
                          color="white"
                          _hover={{}}>
                          Quote
                        </Button>
                      </FormControl>
                    </VStack>
                  </Box>
                </Box>
              </WrapItem>
            </Wrap>
          </Box>
        </Box>
      </Flex>
    </Container>
  );
}

export default App;
